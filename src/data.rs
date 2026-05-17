use anyhow::Result;
use chrono::{DateTime, Local, NaiveDate, TimeDelta};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// 一条番茄钟会话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub started_at: DateTime<Local>,
    pub duration_secs: u64,
    pub task: Option<String>,
    pub status: String,
    pub kind: String,
}

/// 今日统计摘要
#[derive(Debug, Default)]
pub struct Stats {
    pub total_focus_secs: u64,
    pub completed_pomodoros: u32,
    pub interrupted: u32,
}

/// 热力图数据：过去 365 天每天的专注秒数
#[derive(Debug, Default)]
pub struct HeatmapData {
    /// 按日期排序的 (date, seconds) 列表
    pub days: Vec<(NaiveDate, u64)>,
}

/// 数据目录: ~/.local/share/termato/
fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("termato")
}

pub fn history_path() -> PathBuf {
    data_dir().join("history.log")
}

/// 任务队列文件路径
pub fn queue_path() -> PathBuf {
    data_dir().join("queue.txt")
}

fn ensure_data_dir() -> Result<()> {
    let dir = data_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

/// 追加一条会话记录（JSON Lines）
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
            if record.started_at.date_naive() == today && record.kind == "focus" {
                stats.total_focus_secs += record.duration_secs;
                if record.status == "completed" {
                    stats.completed_pomodoros += 1;
                } else {
                    stats.interrupted += 1;
                }
            }
        }
    }
    Ok(stats)
}

/// 扫描全部历史记录，聚合过去 365 天每天的专注秒数
pub fn year_heatmap() -> Result<HeatmapData> {
    let path = history_path();
    if !path.exists() {
        return Ok(HeatmapData::default());
    }

    let cutoff = Local::now().date_naive() - TimeDelta::days(365);
    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let mut map: HashMap<NaiveDate, u64> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(record) = serde_json::from_str::<SessionRecord>(&line) {
            let date = record.started_at.date_naive();
            if date >= cutoff && record.kind == "focus" {
                *map.entry(date).or_insert(0) += record.duration_secs;
            }
        }
    }

    // 生成连续 365 天的列表（无数据的日期填 0）
    let mut days = Vec::new();
    let today = Local::now().date_naive();
    let mut d = cutoff;
    while d <= today {
        let secs = *map.get(&d).unwrap_or(&0);
        days.push((d, secs));
        d += TimeDelta::days(1);
    }

    Ok(HeatmapData { days })
}

/// 追加一个任务到队列文件
pub fn enqueue_task(task: &str) -> Result<()> {
    ensure_data_dir()?;
    let path = queue_path();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{task}")?;
    Ok(())
}

/// 读取并清空任务队列
pub fn drain_queue() -> Result<Vec<String>> {
    let path = queue_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    let tasks: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    // 清空文件
    fs::write(&path, "")?;
    Ok(tasks)
}
