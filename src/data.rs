use anyhow::Result;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// 一条番茄钟会话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// 会话开始时间（本地时区 ISO 8601）
    pub started_at: DateTime<Local>,
    /// 会话持续时间（秒）
    pub duration_secs: u64,
    /// 关联任务名称（可含标签如 #coding）
    pub task: Option<String>,
    /// 完成状态：completed / interrupted / skipped
    pub status: String,
    /// 会话类型：focus / short_break / long_break
    pub kind: String,
}

/// 统计摘要
#[derive(Debug, Default)]
pub struct Stats {
    pub total_focus_secs: u64,
    pub completed_pomodoros: u32,
    pub interrupted: u32,
}

/// 获取数据目录路径: ~/.local/share/termato/
fn data_dir() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("termato")
}

/// 获取历史日志文件路径
pub fn history_path() -> PathBuf {
    data_dir().join("history.log")
}

/// 确保数据目录存在
fn ensure_data_dir() -> Result<()> {
    let dir = data_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// 追加一条会话记录到日志文件（JSON Lines 格式）
pub fn append_record(record: &SessionRecord) -> Result<()> {
    ensure_data_dir()?;
    let path = history_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let mut line = serde_json::to_string(record)?;
    line.push('\n');
    file.write_all(line.as_bytes())?;
    Ok(())
}

/// 读取今天的所有记录并汇总统计
pub fn today_stats() -> Result<Stats> {
    let path = history_path();
    if !path.exists() {
        return Ok(Stats::default());
    }
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let today = Local::now().date_naive();
    let mut stats = Stats::default();

    for line in reader.lines() {
        let line = line?;
        if let Ok(record) = serde_json::from_str::<SessionRecord>(&line) {
            if record.started_at.date_naive() == today {
                match record.kind.as_str() {
                    "focus" => {
                        stats.total_focus_secs += record.duration_secs;
                        if record.status == "completed" {
                            stats.completed_pomodoros += 1;
                        } else {
                            stats.interrupted += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(stats)
}
