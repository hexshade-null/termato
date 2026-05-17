mod config;
mod context;
mod data;
mod hooks;
mod timer;
mod ui;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use hooks::HookEvent;
use notify_rust::Notification;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

// ── CLI 定义 ──────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "termato")]
#[command(about = "A beautiful terminal Pomodoro timer", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动番茄钟 TUI 界面
    Start {
        #[arg(short, long)]
        task: Option<String>,
    },
    /// 显示专注统计
    Stats {
        /// 显示年度热力图
        #[arg(long)]
        heatmap: bool,
    },
    /// 添加任务到队列
    Add {
        /// 任务名称
        task: String,
    },
    /// 生成 Shell 补全脚本
    Completion {
        shell: clap_complete::Shell,
    },
}

// ── 通知 ─────────────────────────────────────────────────

fn send_notification(title: &str, body: &str, cfg: &config::Config) {
    if cfg.notification.desktop {
        if let Err(e) = Notification::new().summary(title).body(body).show() {
            eprintln!("[termato] 桌面通知发送失败: {e}");
        }
    }
    if cfg.notification.sound {
        print!("\x07");
    }
}

// ── TUI 主循环 ────────────────────────────────────────────

fn run_tui(task_name: Option<String>) -> Result<()> {
    let cfg = config::load_config().unwrap_or_else(|e| {
        eprintln!("[termato] 配置加载失败，使用默认值: {e}");
        config::Config::default()
    });

    // Git 自动感知：若没有手动指定 task，尝试从 cwd 获取仓库信息
    let task_name = task_name.or_else(|| {
        let cwd = std::env::current_dir().ok()?;
        let (repo, branch) = context::detect_git_info(&cwd)?;
        Some(format!("{repo}: {branch}"))
    });

    // 读取任务队列
    let queue = data::drain_queue().unwrap_or_default();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut app = timer::PomodoroTimer::new(cfg.clone(), task_name, queue);
    let tick_interval = Duration::from_millis(200);
    let mut ghost_mode = cfg.ui.ghost_mode;
    let mut tick_count: u64 = 0;
    let mut confirm_quit = false;

    loop {
        // 渲染
        terminal.draw(|f| {
            ui::draw(f, &app, &cfg.theme, ghost_mode, tick_count);
            if confirm_quit {
                let accent = match app.phase {
                    timer::Phase::Focus => ui::parse_color_cfg(&cfg.theme.focus_color),
                    _ => ui::parse_color_cfg(&cfg.theme.break_color),
                };
                ui::draw_confirm(f, accent);
            }
        })?;

        tick_count += 1;

        // 导出状态文件
        context::write_status_file(
            &cfg.ui.status_file,
            &app.phase,
            app.remaining().as_secs(),
            app.task_name.as_deref(),
            &app.state,
        );

        // 终端标题联动
        context::set_terminal_title(&app.phase, app.task_name.as_deref());

        // 计时器 tick
        let (completed, hook_evt) = app.tick();
        if completed {
            let label = app.phase_label();
            send_notification("termato", &format!("Phase complete! Next: {label}"), &cfg);
        }
        // 触发钩子
        if let Some(evt) = hook_evt {
            fire_hook_for_event(&cfg, &evt, app.task_name.as_deref());
        }

        // 按键检测
        if event::poll(tick_interval)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // 确认对话框中只处理 y/n/Esc
                if confirm_quit {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => break,
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            confirm_quit = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // 专注运行中需要确认
                        if app.state == timer::State::Running
                            && app.phase == timer::Phase::Focus
                        {
                            confirm_quit = true;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Char(' ') => app.toggle_pause(),
                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Char('s') => {
                        let (_, hook_evt) = app.skip();
                        if let Some(evt) = hook_evt {
                            fire_hook_for_event(&cfg, &evt, app.task_name.as_deref());
                        }
                    }
                    KeyCode::Char('g') => ghost_mode = !ghost_mode,
                    KeyCode::Enter => {
                        if app.state == timer::State::Idle {
                            app.start();
                            // 触发 on_start 钩子
                            if let Some(ref cmd) = cfg.hooks.on_start {
                                hooks::fire_hook(cmd, HookEvent::Start, app.task_name.as_deref());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 退出清理
    if app.state != timer::State::Idle {
        app.record_session_on_quit();
    }
    context::clear_status_file(&cfg.ui.status_file);
    // 恢复终端标题
    print!("\x1b]0;\x07");

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// 根据事件类型触发对应的钩子
fn fire_hook_for_event(cfg: &config::Config, evt: &HookEvent, task: Option<&str>) {
    let cmd = match evt {
        HookEvent::Complete => cfg.hooks.on_complete.as_deref(),
        HookEvent::Break => cfg.hooks.on_break.as_deref(),
        HookEvent::Start => cfg.hooks.on_start.as_deref(),
    };
    if let Some(cmd) = cmd {
        hooks::fire_hook(cmd, evt.clone(), task);
    }
}

// ── 入口 ──────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start { task }) => {
            run_tui(task)?;
        }
        Some(Commands::Stats { heatmap }) => {
            if heatmap {
                let data = data::year_heatmap()?;
                ui::print_heatmap(&data);
            } else {
                let stats = data::today_stats()?;
                ui::print_stats(&stats);
            }
        }
        Some(Commands::Add { task }) => {
            data::enqueue_task(&task)?;
            println!("Task queued: {task}");
        }
        Some(Commands::Completion { shell }) => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "termato", &mut io::stdout());
        }
        None => {
            run_tui(None)?;
        }
    }

    Ok(())
}
