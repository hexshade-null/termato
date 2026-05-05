mod config;
mod data;
mod timer;
mod ui;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
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
        /// 关联的任务名称（支持标签，如 #coding）
        #[arg(short, long)]
        task: Option<String>,
    },
    /// 显示今日专注统计
    Stats,
    /// 生成 Shell 补全脚本
    Completion {
        /// 目标 Shell 类型 (bash, zsh, fish, elvish, powershell)
        shell: clap_complete::Shell,
    },
}

// ── 通知辅助 ──────────────────────────────────────────────

fn send_notification(title: &str, body: &str, config: &config::Config) {
    if config.notification.desktop {
        if let Err(e) = Notification::new()
            .summary(title)
            .body(body)
            .show()
        {
            eprintln!("[termato] 桌面通知发送失败: {e}");
        }
    }
    if config.notification.sound {
        // 终端响铃
        print!("\x07");
    }
}

// ── TUI 主循环 ────────────────────────────────────────────

fn run_tui(task_name: Option<String>) -> Result<()> {
    // 加载配置
    let cfg = config::load_config().unwrap_or_else(|e| {
        eprintln!("[termato] 配置加载失败，使用默认值: {e}");
        config::Config::default()
    });

    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建计时器（初始处于 Idle，需要用户按 Enter 启动）
    let mut app = timer::PomodoroTimer::new(cfg.clone(), task_name);
    let tick_interval = Duration::from_millis(200);

    // 主事件循环
    loop {
        // 渲染
        terminal.draw(|f| ui::draw(f, &app, &cfg.theme))?;

        // 检查计时器完成
        if app.tick() {
            let label = app.phase_label();
            send_notification(
                "termato",
                &format!("Phase complete! Next: {label}"),
                &cfg,
            );
        }

        // 非阻塞按键检测
        if event::poll(tick_interval)? {
            if let Event::Key(key) = event::read()? {
                // 只处理按下事件，忽略释放
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') => app.toggle_pause(),
                    KeyCode::Char('r') => app.reset(),
                    KeyCode::Char('s') => app.skip(),
                    KeyCode::Enter => {
                        if app.state == timer::State::Idle {
                            app.start();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 退出前记录未完成的会话
    if app.state != timer::State::Idle {
        app.record_session_on_quit();
    }

    // 恢复终端
    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// ── 入口 ──────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Start { task }) => {
            run_tui(task)?;
        }
        Some(Commands::Stats) => {
            let stats = data::today_stats()?;
            ui::print_stats(&stats);
        }
        Some(Commands::Completion { shell }) => {
            let mut cmd = Cli::command();
            let name = "termato".to_string();
            clap_complete::generate(shell, &mut cmd, &name, &mut io::stdout());
        }
        None => {
            // 无子命令时直接启动 TUI
            run_tui(None)?;
        }
    }

    Ok(())
}
