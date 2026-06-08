use crate::config::ThemeConfig;
use crate::data::HeatmapData;
use crate::i18n::*;
use crate::timer::{Phase, PomodoroTimer, State};
use chrono::Datelike;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};

// ── 颜色解析 ─────────────────────────────────────────────

pub fn parse_color_cfg(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        "yellow" => Color::Yellow,
        "white" => Color::White,
        "black" => Color::Black,
        hex if hex.starts_with('#') && hex.len() == 7 => {
            let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(255);
            Color::Rgb(r, g, b)
        }
        _ => Color::White,
    }
}

fn accent_color(phase: &Phase, theme: &ThemeConfig) -> Color {
    match phase {
        Phase::Focus => parse_color_cfg(&theme.focus_color),
        Phase::ShortBreak => parse_color_cfg(&theme.break_color),
        Phase::LongBreak => parse_color_cfg(&theme.long_break_color),
    }
}

// ── 大号 ASCII 数字 ──────────────────────────────────────

fn big_digit(ch: char) -> [&'static str; 5] {
    match ch {
        '0' => [" ██████ ","██    ██","██    ██","██    ██"," ██████ "],
        '1' => ["    ██  ","    ██  ","    ██  ","    ██  ","    ██  "],
        '2' => [" ██████ ","     ██ "," ██████ ","██      "," ██████ "],
        '3' => [" ██████ ","     ██ "," ██████ ","     ██ "," ██████ "],
        '4' => ["██    ██","██    ██"," ███████","      ██","      ██"],
        '5' => [" ███████","██      "," ██████ ","      ██"," ██████ "],
        '6' => [" ██████ ","██      "," ██████ ","██    ██"," ██████ "],
        '7' => [" ███████","     ██ ","    ██  ","   ██   ","   ██   "],
        '8' => [" ██████ ","██    ██"," ██████ ","██    ██"," ██████ "],
        '9' => [" ██████ ","██    ██"," ███████","      ██"," ██████ "],
        ':' => ["        ","   ██   ","        ","   ██   ","        "],
        _ => ["        ","        ","        ","        ","        "],
    }
}

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

// ── Braille 精细进度条 ───────────────────────────────────

const BRAILLE_BASE: u32 = 0x2800;

fn braille_bar(progress: f64, width: usize) -> String {
    let total_dots = width * 2;
    let filled = ((progress * total_dots as f64).round() as usize).min(total_dots);
    let mut out = String::with_capacity(width);
    for col_pair in (0..total_dots).step_by(2) {
        let mut bits: u32 = 0;
        if col_pair < filled {
            bits |= 0x01 | 0x02 | 0x04 | 0x40;
        }
        if col_pair + 1 < filled {
            bits |= 0x08 | 0x10 | 0x20 | 0x80;
        }
        out.push(char::from_u32(BRAILLE_BASE + bits).unwrap_or('⠀'));
    }
    out
}

// ── 休息期呼吸动画 ──────────────────────────────────────

fn breath_frame(tick: u64) -> Vec<String> {
    let frames = [
        vec!["         ","    .    ","   ( )   ","    '    ","         "],
        vec!["         ","   .-.   ","  (   )  ","   '-'   ","         "],
        vec!["    _    ","  .\\ /.  "," (  O  ) ","  '/ \\'  ","    '    "],
        vec!["   ___   ","  / . \\  "," (  |  ) ","  \\ ' /  ","   '''   "],
        vec!["  _____  "," /     \\ ","(  ~~~  )"," \\_____/ ","         "],
        vec!["   ___   ","  / . \\  "," (  |  ) ","  \\ ' /  ","   '''   "],
        vec!["    _    ","  .\\ /.  "," (  O  ) ","  '/ \\'  ","    '    "],
        vec!["         ","   .-.   ","  (   )  ","   '-'   ","         "],
    ];
    frames[(tick % 8) as usize].iter().map(|s| s.to_string()).collect()
}

// ── 阶段标签 ─────────────────────────────────────────────

fn phase_text(phase: &Phase) -> &'static str {
    match phase {
        Phase::Focus => phase_focus(),
        Phase::ShortBreak => phase_short_break(),
        Phase::LongBreak => phase_long_break(),
    }
}

// ── 主渲染函数 ──────────────────────────────────────────

pub fn draw(f: &mut Frame, timer: &PomodoroTimer, theme: &ThemeConfig, ghost_mode: bool, tick_count: u64) {
    let size = f.area();
    let accent = accent_color(&timer.phase, theme);
    let digit_color = parse_color_cfg(&theme.digit_color);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(size);

    // ── 标题栏 ──
    let state_text = match timer.state {
        State::Running => state_running(),
        State::Paused => state_paused(),
        State::Idle => state_idle(),
    };
    let g_tag = if ghost_mode { ghost_tag() } else { "" };
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" termato ", Style::default().fg(Color::Black).bg(accent).bold()),
        Span::raw("  "),
        Span::styled(
            format!("{} — {}{g_tag}", phase_text(&timer.phase), state_text),
            Style::default().fg(accent).bold(),
        ),
    ]));
    f.render_widget(title, chunks[0]);

    // ── 中央区域 ──
    if ghost_mode {
        let msg = match timer.phase {
            Phase::Focus => ghost_focus(),
            _ => ghost_break(),
        };
        let ghost = Paragraph::new(msg)
            .style(Style::default().fg(accent))
            .alignment(Alignment::Center);
        f.render_widget(ghost, chunks[1]);
    } else if matches!(timer.phase, Phase::ShortBreak | Phase::LongBreak) {
        let frame = breath_frame(tick_count);
        let anim = Paragraph::new(
            frame.iter().map(|l| Line::from(l.clone())).collect::<Vec<_>>(),
        )
        .style(Style::default().fg(accent))
        .alignment(Alignment::Center);
        f.render_widget(anim, chunks[1]);
    } else {
        let rem = timer.remaining();
        let mins = (rem.as_secs() / 60) as u32;
        let secs = (rem.as_secs() % 60) as u32;
        let time_str = format!("{mins:02}:{secs:02}");
        let big_lines = render_big_time(&time_str);
        let digit_p = Paragraph::new(
            big_lines.iter().map(|l| Line::from(l.clone())).collect::<Vec<_>>(),
        )
        .style(Style::default().fg(digit_color))
        .alignment(Alignment::Center);
        f.render_widget(digit_p, chunks[1]);
    }

    // ── 进度条 ──
    let progress = timer.progress();
    let bar_width = (chunks[2].width.saturating_sub(4)) as usize;
    if bar_width > 4 {
        let bar_str = braille_bar(progress, bar_width.min(50));
        let bar_line = Paragraph::new(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(bar_str, Style::default().fg(accent)),
            Span::styled(format!(" {:5.1}%", progress * 100.0), Style::default().fg(accent)),
        ]))
        .style(Style::default().bg(Color::DarkGray));
        f.render_widget(bar_line, chunks[2]);
    } else {
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(accent).bg(Color::DarkGray))
            .ratio(progress);
        f.render_widget(gauge, chunks[2]);
    }

    // ── 任务名 + 队列 ──
    let task_text = timer.task_name.as_deref().unwrap_or(label_no_task());
    let queue_hint = if timer.pending_count() > 0 {
        format!("(+{}){}", timer.pending_count(), label_queued())
    } else {
        String::new()
    };
    let task = Paragraph::new(Line::from(vec![
        Span::styled(label_task(), Style::default().fg(Color::DarkGray)),
        Span::styled(task_text, Style::default().fg(Color::Yellow)),
        Span::styled(&queue_hint, Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(task, chunks[3]);

    // ── 番茄计数 ──
    let count = timer.completed_count();
    let count_bar = Paragraph::new(Line::from(vec![
        Span::styled("🍅 ", Style::default()),
        Span::styled(
            format!("{}{count}  ", label_completed()),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(count_bar, chunks[4]);

    // ── 快捷键提示 ──
    let help = Paragraph::new(Line::from(vec![
        Span::styled("[Space]", Style::default().fg(accent).bold()),
        Span::raw(key_pause()),
        Span::styled("[R]", Style::default().fg(accent).bold()),
        Span::raw(key_reset()),
        Span::styled("[S]", Style::default().fg(accent).bold()),
        Span::raw(key_skip()),
        Span::styled("[Enter]", Style::default().fg(accent).bold()),
        Span::raw(key_start()),
        Span::styled("[G]", Style::default().fg(accent).bold()),
        Span::raw(key_ghost()),
        Span::styled("[Q]", Style::default().fg(accent).bold()),
        Span::raw(key_quit()),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[5]);
}

// ── 确认对话框 ───────────────────────────────────────────

pub fn draw_confirm(f: &mut Frame, _accent: Color) {
    let area = centered_rect(40, 5, f.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(confirm_title())
        .style(Style::default().bg(Color::DarkGray));
    let msg = Paragraph::new(Line::from(vec![
        Span::styled(confirm_message(), Style::default().fg(Color::Yellow).bold()),
    ]))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(msg, area);

    let btn_area = Rect { y: area.y + 3, ..area };
    let btn = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[y]", Style::default().fg(Color::Red).bold()),
        Span::raw(format!(" {}  ", key_quit().trim())),
        Span::styled("[n/Esc]", Style::default().fg(Color::Green).bold()),
        Span::raw(confirm_no().trim().strip_prefix("[n/Esc]").unwrap_or(" Cancel")),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(Color::DarkGray));
    f.render_widget(btn, btn_area);
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.width.saturating_sub(width) / 2;
    let y = r.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}

// ── 统计打印 ─────────────────────────────────────────────

pub fn print_stats(stats: &crate::data::Stats) {
    let hours = stats.total_focus_secs / 3600;
    let mins = (stats.total_focus_secs % 3600) / 60;
    let secs = stats.total_focus_secs % 60;

    let title = stats_title();
    let title_w = unicode_width(title);
    let w = title_w.max(28);
    let top = format!("┌{}┐", "─".repeat(w + 2));
    let mid = format!("│ {}{} │", title, " ".repeat(w - title_w));
    let div = format!("├{}┤", "─".repeat(w + 2));
    let bot = format!("└{}┘", "─".repeat(w + 2));

    let ft = stats_focus_time();
    let sc = stats_completed();
    let si = stats_interrupted();
    let ft_w = unicode_width(ft);
    let sc_w = unicode_width(sc);
    let si_w = unicode_width(si);

    println!("{top}");
    println!("{mid}");
    println!("{div}");
    println!("│ {}{:02}h {:02}m {:02}s{} │", ft, hours, mins, secs, " ".repeat(w - ft_w - 10));
    println!("│ {}{}{} │", sc, stats.completed_pomodoros, " ".repeat(w - sc_w - 12));
    println!("│ {}{}{} │", si, stats.interrupted, " ".repeat(w - si_w - 10));
    println!("{bot}");
}

/// 计算 Unicode 字符串的终端显示宽度（CJK 字符算 2 列）
fn unicode_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    s.width()
}

// ── 热力图 ──────────────────────────────────────────────

pub fn print_heatmap(data: &HeatmapData) {
    if data.days.is_empty() {
        println!("{}", heatmap_empty());
        return;
    }

    let levels = [' ', '░', '▒', '▓', '█'];
    let max_secs: f64 = 4.0 * 3600.0;

    let first_date = data.days[0].0;
    let first_weekday = first_date.weekday().num_days_from_monday() as usize;
    let total_weeks = (data.days.len() + first_weekday + 6) / 7 + 1;
    let mut grid: Vec<Vec<char>> = vec![vec![' '; total_weeks]; 7];

    for (date, secs) in &data.days {
        let offset = (*date - first_date).num_days() as usize;
        let pos = first_weekday + offset;
        let week = pos / 7;
        let weekday = pos % 7;
        let ratio = (*secs as f64 / max_secs).min(1.0);
        let level = (ratio * 4.0).round() as usize;
        if weekday < 7 && week < total_weeks {
            grid[weekday][week] = levels[level.min(4)];
        }
    }

    let month_names = match current_lang() {
        Lang::ZhCn | Lang::ZhTw => ["1月","2月","3月","4月","5月","6月","7月","8月","9月","10月","11月","12月"],
        _ => ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"],
    };

    print!("     ");
    let mut last_month = 255u32;
    for w in 0..total_weeks {
        if let Some((d, _)) = data.days.get(w * 7) {
            let m = d.month() as u32;
            if m != last_month {
                print!("{}", month_names[m as usize - 1]);
                last_month = m;
            } else {
                print!("   ");
            }
        }
    }
    println!();

    let day_labels = match current_lang() {
        Lang::ZhCn | Lang::ZhTw => ["周一", "周二", "周三", "周四", "周五", "周六", "周日"],
        _ => ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    };

    for weekday in 0..7 {
        print!("{} ", day_labels[weekday]);
        for week in 0..total_weeks {
            print!("{}", grid[weekday][week]);
        }
        println!();
    }

    println!("\n  {}", heatmap_legend());
}
