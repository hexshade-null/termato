use crate::i18n::*;
use crate::timer::{Phase, State};
use std::fs;
use std::path::Path;

/// 检测给定目录是否位于 Git 仓库内
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

/// 写入状态文件
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
    let _ = fs::write(path, content);
}

/// 清除状态文件
pub fn clear_status_file(path: &str) {
    let _ = fs::remove_file(path);
}

/// 设置终端窗口标题（使用 i18n）
pub fn set_terminal_title(phase: &Phase, task: Option<&str>) {
    let phase_label = match phase {
        Phase::Focus => title_focus(),
        Phase::ShortBreak => title_short_break(),
        Phase::LongBreak => title_long_break(),
    };
    let title = match task {
        Some(t) => format!("[termato] {phase_label} - {t}"),
        None => format!("[termato] {phase_label}"),
    };
    print!("\x1b]0;{title}\x07");
}
