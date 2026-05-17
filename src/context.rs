use crate::timer::{Phase, State};
use std::fs;
use std::path::Path;

/// 检测给定目录是否位于 Git 仓库内。
/// 如果是，返回 (仓库名, 当前分支名)。
pub fn detect_git_info(dir: &Path) -> Option<(String, String)> {
    let repo = git2::Repository::discover(dir).ok()?;
    let repo_name = repo
        .workdir()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from))
        .unwrap_or_else(|| "detached".to_string());
    Some((repo_name, branch))
}

/// 将当前状态写入状态文件，供 tmux/Polybar 等外部工具读取。
/// 格式：每行一个 key=value，如：
///   phase=focus
///   remaining=842
///   task=写代码
pub fn write_status_file(path: &str, phase: &Phase, remaining_secs: u64, task: Option<&str>, state: &State) {
    let state_name = match state {
        State::Idle => "idle",
        State::Running => "running",
        State::Paused => "paused",
    };
    let phase_name = match phase {
        Phase::Focus => "focus",
        Phase::ShortBreak => "short_break",
        Phase::LongBreak => "long_break",
    };
    let content = format!(
        "phase={phase_name}\nremaining={remaining_secs}\ntask={}\nstate={state_name}\n",
        task.unwrap_or("")
    );
    // 静默写入
    let _ = fs::write(path, content);
}

/// 清除状态文件（程序退出时调用）
pub fn clear_status_file(path: &str) {
    let _ = fs::remove_file(path);
}

/// 设置终端窗口标题（通过 ANSI escape sequence）
/// 格式示例："[termato] 专注中 - 写代码"
pub fn set_terminal_title(phase: &Phase, task: Option<&str>) {
    let phase_label = match phase {
        Phase::Focus => "专注中",
        Phase::ShortBreak => "短休息",
        Phase::LongBreak => "长休息",
    };
    let title = match task {
        Some(t) => format!("[termato] {phase_label} - {t}"),
        None => format!("[termato] {phase_label}"),
    };
    // OSC escape: \x1b]0;...\x07
    print!("\x1b]0;{title}\x07");
}
