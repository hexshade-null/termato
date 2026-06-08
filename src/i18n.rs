use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::OnceLock;

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    #[default]
    En,
    ZhCn,
    ZhTw,
}

impl FromStr for Lang {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Lang::En),
            "zh_cn" | "zh-cn" | "zhcn" => Ok(Lang::ZhCn),
            "zh_tw" | "zh-tw" | "zhtw" => Ok(Lang::ZhTw),
            other => Err(format!("Unknown language: {other} (supported: en, zh_cn, zh_tw)")),
        }
    }
}

// ── 所有翻译字符串 ──────────────────────────────────────

static LANG: OnceLock<Lang> = OnceLock::new();

pub fn set_lang(lang: Lang) {
    let _ = LANG.set(lang);
}

pub fn current_lang() -> Lang {
    *LANG.get().unwrap_or(&Lang::En)
}

macro_rules! tr {
    ($en:expr, $zh_cn:expr, $zh_tw:expr) => {
        match $crate::i18n::current_lang() {
            $crate::i18n::Lang::En => $en,
            $crate::i18n::Lang::ZhCn => $zh_cn,
            $crate::i18n::Lang::ZhTw => $zh_tw,
        }
    };
}

// ── TUI 界面 ─────────────────────────────────────────────

pub fn phase_focus() -> &'static str { tr!("Focus", "专注", "專注") }
pub fn phase_short_break() -> &'static str { tr!("Short Break", "短休息", "短休息") }
pub fn phase_long_break() -> &'static str { tr!("Long Break", "长休息", "長休息") }

pub fn state_running() -> &'static str { tr!("● RUNNING", "● 运行中", "● 執行中") }
pub fn state_paused() -> &'static str { tr!("❚❚ PAUSED", "❚❚ 已暂停", "❚❚ 已暫停") }
pub fn state_idle() -> &'static str { tr!("○ IDLE", "○ 空闲", "○ 閒置") }

pub fn ghost_focus() -> &'static str { tr!("Focus in progress...", "专注进行中...", "專注進行中...") }
pub fn ghost_break() -> &'static str { tr!("Take a breath...", "放松一下...", "放鬆一下...") }

pub fn label_task() -> &'static str { tr!("Task: ", "任务: ", "任務: ") }
pub fn label_completed() -> &'static str { tr!("Completed: ", "已完成: ", "已完成: ") }
pub fn label_no_task() -> &'static str { tr!("No task assigned", "未指定任务", "未指定任務") }
pub fn label_queued() -> &'static str { tr!(" queued", " 待处理", " 待處理") }

pub fn key_pause() -> &'static str { tr!(" Pause ", " 暂停 ", " 暫停 ") }
pub fn key_reset() -> &'static str { tr!(" Reset ", " 重置 ", " 重置 ") }
pub fn key_skip() -> &'static str { tr!(" Skip ", " 跳过 ", " 跳過 ") }
pub fn key_start() -> &'static str { tr!(" Start ", " 开始 ", " 開始 ") }
pub fn key_ghost() -> &'static str { tr!(" Ghost ", " 幽灵 ", " 幽靈 ") }
pub fn key_quit() -> &'static str { tr!(" Quit ", " 退出 ", " 退出 ") }
pub fn ghost_tag() -> &'static str { tr!(" [GHOST]", " [幽灵]", " [幽靈]") }

// ── 确认对话框 ───────────────────────────────────────────

pub fn confirm_title() -> &'static str { tr!(" Confirm ", " 确认 ", " 確認 ") }
pub fn confirm_message() -> &'static str { tr!(" Abandon current focus?", " 放弃当前专注？", " 放棄當前專注？") }
pub fn confirm_yes() -> &'static str { tr!("[y] Quit", "[y] 退出", "[y] 退出") }
pub fn confirm_no() -> &'static str { tr!("[n/Esc] Cancel", "[n/Esc] 取消", "[n/Esc] 取消") }

// ── 通知 ─────────────────────────────────────────────────

pub fn notify_complete(phase_next: &str) -> String {
    match current_lang() {
        Lang::En => format!("Phase complete! Next: {phase_next}"),
        Lang::ZhCn => format!("阶段完成！下一阶段: {phase_next}"),
        Lang::ZhTw => format!("階段完成！下一階段: {phase_next}"),
    }
}

// ── 统计 ─────────────────────────────────────────────────

pub fn stats_title() -> &'static str { tr!("termato — Today's Stats", "termato — 今日统计", "termato — 今日統計") }
pub fn stats_focus_time() -> &'static str { tr!("Focus time ", "专注时长 ", "專注時長 ") }
pub fn stats_completed() -> &'static str { tr!("Completed  ", "已完成    ", "已完成    ") }
pub fn stats_pomodoros() -> &'static str { tr!(" pomodoros", " 个番茄", " 個番茄") }
pub fn stats_interrupted() -> &'static str { tr!("Interrupted", "已中断    ", "已中斷    ") }

// ── 热力图 ──────────────────────────────────────────────

pub fn heatmap_empty() -> &'static str { tr!("No history data yet.", "暂无历史数据。", "暫無歷史數據。") }
pub fn heatmap_legend() -> &'static str {
    tr!("Less ░ ▒ ▓ █ More   (each █ ≈ 4h focus)",
        "少 ░ ▒ ▓ █ 多   (每个 █ ≈ 4小时专注)",
        "少 ░ ▒ ▓ █ 多   (每個 █ ≈ 4小時專注)")
}

// ── CLI ──────────────────────────────────────────────────

pub fn cli_task_queued(task: &str) -> String {
    match current_lang() {
        Lang::En => format!("Task queued: {task}"),
        Lang::ZhCn => format!("任务已入队: {task}"),
        Lang::ZhTw => format!("任務已加入佇列: {task}"),
    }
}

// ── 终端标题 ─────────────────────────────────────────────

pub fn title_focus() -> &'static str { tr!("Focusing", "专注中", "專注中") }
pub fn title_short_break() -> &'static str { tr!("Short Break", "短休息", "短休息") }
pub fn title_long_break() -> &'static str { tr!("Long Break", "长休息", "長休息") }
