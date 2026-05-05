use crate::config::ThemeConfig;
use crate::timer::{Phase, PomodoroTimer, State};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

/// 将配置中的颜色字符串解析为 ratatui Color
fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        "yellow" => Color::Yellow,
        "white" => Color::White,
        "black" => Color::Black,
        // 尝试解析 hex 如 "#ff5555"
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(255);
            Color::Rgb(r, g, b)
        }
        _ => Color::White,
    }
}

/// 根据当前阶段获取主题强调色
fn accent_color(phase: &Phase, theme: &ThemeConfig) -> Color {
    match phase {
        Phase::Focus => parse_color(&theme.focus_color),
        Phase::ShortBreak => parse_color(&theme.break_color),
        Phase::LongBreak => parse_color(&theme.long_break_color),
    }
}

/// 使用 ASCII block 字符绘制大号数字
fn big_digit(ch: char) -> [&'static str; 5] {
    match ch {
        '0' => [
            " ██████ ",
            "██    ██",
            "██    ██",
            "██    ██",
            " ██████ ",
        ],
        '1' => [
            "    ██  ",
            "    ██  ",
            "    ██  ",
            "    ██  ",
            "    ██  ",
        ],
        '2' => [
            " ██████ ",
            "     ██ ",
            " ██████ ",
            "██      ",
            " ██████ ",
        ],
        '3' => [
            " ██████ ",
            "     ██ ",
            " ██████ ",
            "     ██ ",
            " ██████ ",
        ],
        '4' => [
            "██    ██",
            "██    ██",
            " ███████",
            "      ██",
            "      ██",
        ],
        '5' => [
            " ███████",
            "██      ",
            " ██████ ",
            "      ██",
            " ██████ ",
        ],
        '6' => [
            " ██████ ",
            "██      ",
            " ██████ ",
            "██    ██",
            " ██████ ",
        ],
        '7' => [
            " ███████",
            "     ██ ",
            "    ██  ",
            "   ██   ",
            "   ██   ",
        ],
        '8' => [
            " ██████ ",
            "██    ██",
            " ██████ ",
            "██    ██",
            " ██████ ",
        ],
        '9' => [
            " ██████ ",
            "██    ██",
            " ███████",
            "      ██",
            " ██████ ",
        ],
        ':' => [
            "        ",
            "   ██   ",
            "        ",
            "   ██   ",
            "        ",
        ],
        _ => [
            "        ",
            "        ",
            "        ",
            "        ",
            "        ",
        ],
    }
}

/// 将 MM:SS 格式的字符串转为大号 ASCII art 行（5 行）
fn render_big_time(mmss: &str) -> Vec<String> {
    let chars: Vec<char> = mmss.chars().collect();
    let mut lines = vec![String::new(); 5];
    for ch in &chars {
        let glyph = big_digit(*ch);
        for (i, row) in glyph.iter().enumerate() {
            lines[i].push_str(row);
        }
    }
    lines
}

/// 主渲染函数
pub fn draw(f: &mut Frame, timer: &PomodoroTimer, theme: &ThemeConfig) {
    let size = f.area();
    let accent = accent_color(&timer.phase, theme);
    let digit_color = parse_color(&theme.digit_color);

    // 整体布局：上 → 中（倒计时）→ 进度条 → 任务名 → 快捷键
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 顶部标题栏
            Constraint::Min(8),    // 大号倒计时
            Constraint::Length(3),  // 进度条
            Constraint::Length(2),  // 任务名
            Constraint::Length(3),  // 番茄计数
            Constraint::Length(2),  // 快捷键提示
        ])
        .split(size);

    // ── 标题栏 ──
    let state_text = match timer.state {
        State::Running => "● RUNNING",
        State::Paused => "❚❚ PAUSED",
        State::Idle => "○ IDLE",
    };
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " termato ",
            Style::default().fg(Color::Black).bg(accent).bold(),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} — {}", timer.phase_label(), state_text),
            Style::default().fg(accent).bold(),
        ),
    ]));
    f.render_widget(title, chunks[0]);

    // ── 大号倒计时 ──
    let rem = timer.remaining();
    let mins = (rem.as_secs() / 60) as u32;
    let secs = (rem.as_secs() % 60) as u32;
    let time_str = format!("{mins:02}:{secs:02}");
    let big_lines = render_big_time(&time_str);

    let digit_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().fg(digit_color).bg(Color::Reset));
    let digit_paragraph = Paragraph::new(
        big_lines
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect::<Vec<_>>(),
    )
    .block(digit_block)
    .alignment(Alignment::Center);
    f.render_widget(digit_paragraph, chunks[1]);

    // ── 进度条 ──
    let progress = timer.progress();
    let gauge_label = format!("{:.0}%", progress * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(accent).bg(Color::DarkGray))
        .ratio(progress)
        .label(gauge_label);
    f.render_widget(gauge, chunks[2]);

    // ── 任务名 ──
    let task_text = timer
        .task_name
        .as_deref()
        .unwrap_or("No task assigned");
    let task = Paragraph::new(Line::from(vec![
        Span::styled(" Task: ", Style::default().fg(Color::DarkGray)),
        Span::styled(task_text, Style::default().fg(Color::Yellow)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(task, chunks[3]);

    // ── 番茄计数 ──
    let count_text = format!(" Completed: {}  ", timer.completed_count());
    let count_bar = Paragraph::new(Line::from(vec![
        Span::styled("🍅 ", Style::default()),
        Span::styled(
            count_text,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(count_bar, chunks[4]);

    // ── 快捷键提示 ──
    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            " [Space]",
            Style::default().fg(accent).bold(),
        ),
        Span::raw(" Pause/Resume "),
        Span::styled("[R]", Style::default().fg(accent).bold()),
        Span::raw(" Reset "),
        Span::styled("[S]", Style::default().fg(accent).bold()),
        Span::raw(" Skip "),
        Span::styled("[Enter]", Style::default().fg(accent).bold()),
        Span::raw(" Start "),
        Span::styled("[Q]", Style::default().fg(accent).bold()),
        Span::raw(" Quit "),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[5]);
}

/// 在终端打印今日统计信息（非 TUI 模式）
pub fn print_stats(stats: &crate::data::Stats) {
    let hours = stats.total_focus_secs / 3600;
    let mins = (stats.total_focus_secs % 3600) / 60;
    let secs = stats.total_focus_secs % 60;

    println!("┌──────────────────────────────┐");
    println!("│     termato — Today's Stats  │");
    println!("├──────────────────────────────┤");
    println!("│  Focus time : {hours:02}h {mins:02}m {secs:02}s       │");
    println!("│  Completed  : {} pomodoros      │", stats.completed_pomodoros);
    println!("│  Interrupted: {}               │", stats.interrupted);
    println!("└──────────────────────────────┘");
}
