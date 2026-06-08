mod config;
mod context;
mod data;
mod hooks;
mod i18n;
mod timer;
mod ui;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use hooks::HookEvent;
use i18n::*;
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
    /// Start the Pomodoro TUI
    Start {
        #[arg(short, long)]
        task: Option<String>,
    },
    /// Show focus statistics
    Stats {
        /// Show yearly heatmap
        #[arg(long)]
        heatmap: bool,
    },
    /// Add a task to the queue
    Add {
        /// Task name
        task: String,
    },
    /// Generate shell completion script
    Completion {
        shell: clap_complete::Shell,
    },
}

// ── 通知 ─────────────────────────────────────────────────

fn send_notification(title: &str, body: &str, cfg: &config::Config) {
    if cfg.notification.desktop {
        if let Err(e) = Notification::new().summary(title).body(body).show() {
            eprintln!("[termato] Notification failed: {e}");
        }
    }
    if cfg.notification.sound {
        print!("\x07");
    }
}

// ── TUI 主循环 ────────────────────────────────────────────

fn run_tui(task_name: Option<String>) -> Result<()> {
    let cfg = config::load_config().unwrap_or_else(|e| {
        eprintln!("[termato] Config load failed, using defaults: {e}");
        config::Config::default()
    });

    // 初始化语言
    i18n::set_lang(cfg.lang());

    // Git 自动感知
    let task_name = task_name.or_else(|| {
        let cwd = std::env::current_dir().ok()?;
        let (repo, branch) = context::detect_git_info(&cwd)?;
        Some(format!("{repo}: {branch}"))
    });

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

        context::write_status_file(
            &cfg.ui.status_file,
            &app.phase,
            app.remaining().as_secs(),
            app.task_name.as_deref(),
            &app.state,
        );

        context::set_terminal_title(&app.phase, app.task_name.as_deref());

        let (completed, hook_evt) = app.tick();
        if completed {
            let label = app.phase_label();
            send_notification("termato", &notify_complete(label), &cfg);
        }
        if let Some(evt) = hook_evt {
            fire_hook_for_event(&cfg, &evt, app.task_name.as_deref());
        }

        if event::poll(tick_interval)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

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

    if app.state != timer::State::Idle {
        app.record_session_on_quit();
    }
    context::clear_status_file(&cfg.ui.status_file);
    print!("\x1b]0;\x07");

    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

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
    // 预加载配置以确定语言（影响 stats/add 等非 TUI 子命令的输出）
    let lang = config::load_config()
        .map(|c| c.lang())
        .unwrap_or_default();
    i18n::set_lang(lang);

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
            println!("{}", cli_task_queued(&task));
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
