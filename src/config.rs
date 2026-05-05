use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 主配置结构体，对应 config.toml 的顶层字段
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub timer: TimerConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
}

/// 计时器相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    /// 专注时长（分钟）
    #[serde(default = "default_focus")]
    pub focus_minutes: u64,
    /// 短休息时长（分钟）
    #[serde(default = "default_short_break")]
    pub short_break_minutes: u64,
    /// 长休息时长（分钟）
    #[serde(default = "default_long_break")]
    pub long_break_minutes: u64,
    /// 每轮长休息前的番茄数
    #[serde(default = "default_rounds")]
    pub rounds_before_long_break: u32,
}

fn default_focus() -> u64 {
    25
}
fn default_short_break() -> u64 {
    5
}
fn default_long_break() -> u64 {
    15
}
fn default_rounds() -> u32 {
    4
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            focus_minutes: default_focus(),
            short_break_minutes: default_short_break(),
            long_break_minutes: default_long_break(),
            rounds_before_long_break: default_rounds(),
        }
    }
}

/// 通知相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 是否发送桌面通知
    #[serde(default = "default_true")]
    pub desktop: bool,
    /// 是否播放提示音
    #[serde(default = "default_true")]
    pub sound: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            desktop: true,
            sound: true,
        }
    }
}

/// 主题配色配置（ratatui 颜色值）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// 专注阶段的强调色
    #[serde(default = "default_focus_color")]
    pub focus_color: String,
    /// 休息阶段的强调色
    #[serde(default = "default_break_color")]
    pub break_color: String,
    /// 长休息阶段的强调色
    #[serde(default = "default_long_break_color")]
    pub long_break_color: String,
    /// 大字倒计时的颜色
    #[serde(default = "default_digit_color")]
    pub digit_color: String,
}

fn default_focus_color() -> String {
    "Red".into()
}
fn default_break_color() -> String {
    "Green".into()
}
fn default_long_break_color() -> String {
    "Cyan".into()
}
fn default_digit_color() -> String {
    "White".into()
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            focus_color: default_focus_color(),
            break_color: default_break_color(),
            long_break_color: default_long_break_color(),
            digit_color: default_digit_color(),
        }
    }
}

/// 获取配置文件路径: ~/.config/termato/config.toml
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("termato")
        .join("config.toml")
}

/// 从文件加载配置；文件不存在则返回默认值
pub fn load_config() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("解析配置文件失败: {}", path.display()))?;
    Ok(config)
}
