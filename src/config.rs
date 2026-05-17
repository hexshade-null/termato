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
    #[serde(default)]
    pub hooks: HookConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

/// 计时器相关配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerConfig {
    #[serde(default = "default_focus")]
    pub focus_minutes: u64,
    #[serde(default = "default_short_break")]
    pub short_break_minutes: u64,
    #[serde(default = "default_long_break")]
    pub long_break_minutes: u64,
    #[serde(default = "default_rounds")]
    pub rounds_before_long_break: u32,
}

fn default_focus() -> u64 { 25 }
fn default_short_break() -> u64 { 5 }
fn default_long_break() -> u64 { 15 }
fn default_rounds() -> u32 { 4 }

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
    #[serde(default = "default_true")]
    pub desktop: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
}

fn default_true() -> bool { true }

impl Default for NotificationConfig {
    fn default() -> Self {
        Self { desktop: true, sound: true }
    }
}

/// 主题配色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_focus_color")]
    pub focus_color: String,
    #[serde(default = "default_break_color")]
    pub break_color: String,
    #[serde(default = "default_long_break_color")]
    pub long_break_color: String,
    #[serde(default = "default_digit_color")]
    pub digit_color: String,
}

fn default_focus_color() -> String { "Red".into() }
fn default_break_color() -> String { "Green".into() }
fn default_long_break_color() -> String { "Cyan".into() }
fn default_digit_color() -> String { "White".into() }

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

/// 钩子脚本配置：事件触发时异步执行外部 Shell 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// 专注开始时执行的命令
    #[serde(default)]
    pub on_start: Option<String>,
    /// 进入休息时执行的命令
    #[serde(default)]
    pub on_break: Option<String>,
    /// 番茄完成时执行的命令
    #[serde(default)]
    pub on_complete: Option<String>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            on_start: None,
            on_break: None,
            on_complete: None,
        }
    }
}

/// UI 行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// 幽灵模式：隐藏倒计时数字，仅显示极简进度条
    #[serde(default)]
    pub ghost_mode: bool,
    /// 状态文件导出路径（用于 tmux/Polybar 等外部工具读取）
    #[serde(default = "default_status_file")]
    pub status_file: String,
}

fn default_status_file() -> String { "/tmp/termato.status".into() }

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            ghost_mode: false,
            status_file: default_status_file(),
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
