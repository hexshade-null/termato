# termato

一个基于 Rust 构建的跨平台终端番茄钟工具。

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)

[English](./README.md)

## 功能特性

### 核心功能
- **TUI 界面** — 大号 ASCII 倒计时、Braille 精细进度条、快捷键提示
- **番茄工作法循环** — 25 分钟专注 → 5 分钟短休息 → 15 分钟长休息（每 4 轮）
- **任务追踪** — 为每次会话关联任务名称（`-t "写文档 #coding"`）
- **可配置** — 通过配置文件自定义时长、配色、通知等
- **数据持久化** — 会话记录以 JSON Lines 格式持久存储
- **桌面通知** — 阶段结束时发送系统级通知
- **跨平台** — 支持 macOS、Linux 和 Windows

### 沉浸式与防干扰
- **幽灵模式** — 隐藏倒计时数字，仅显示极简进度条（按 `G` 切换）
- **防误触退出** — 专注运行中退出时弹出二次确认对话框
- **光标自动隐藏** — TUI 运行期间隐藏终端光标

### 自动化与系统集成
- **钩子脚本** — 在 `on_start`、`on_break`、`on_complete` 事件触发时执行 Shell 命令
- **状态文件导出** — 实时写入状态文件，供 tmux/Polybar 等外部工具读取
- **终端标题联动** — 动态修改终端窗口标题，显示当前阶段和任务名

### 智能任务
- **Git 仓库自动感知** — 在 Git 仓库中启动时自动填充 `"repo: branch"` 作为任务名
- **任务队列** — `termato add "任务A"` 预先添加任务，TUI 中按顺序自动处理

### 极客美学
- **Braille 精细进度条** — Unicode 点阵绘制平滑进度指示器
- **休息期动画** — 休息期间播放呼吸动画，增加趣味性
- **热力图统计** — `termato stats --heatmap` 显示类似 GitHub 的年度专注热力图

## 安装

### 从源码编译

```bash
git clone https://github.com/hexshade-null/termato.git
cd termato
cargo build --release
cargo install --path .
```

### 前置要求

- [Rust](https://rustup.rs/) 1.75 或更高版本

## 使用方法

```bash
termato                              # 启动 TUI
termato start -t "写文档"             # 带任务名启动
termato add "任务A"                   # 添加任务到队列
termato add "任务B"                   # 可添加多个
termato stats                        # 查看今日专注统计
termato stats --heatmap              # 查看年度热力图
termato completion bash              # 生成 Shell 补全脚本
```

### 快捷键

| 按键    | 操作             |
|---------|-----------------|
| Enter   | 开始计时         |
| Space   | 暂停 / 继续       |
| R       | 重置当前阶段      |
| S       | 跳过当前阶段      |
| G       | 切换幽灵模式      |
| Q / Esc | 退出             |

## 配置

复制示例配置并按需修改：

```bash
# macOS / Linux
mkdir -p ~/.config/termato
cp examples/config.toml ~/.config/termato/config.toml

# Windows (PowerShell)
mkdir "$env:APPDATA\termato"
Copy-Item examples/config.toml "$env:APPDATA\termato\config.toml"
```

示例 `config.toml`：

```toml
[timer]
focus_minutes = 25
short_break_minutes = 5
long_break_minutes = 15
rounds_before_long_break = 4

[notification]
desktop = true
sound = true

[theme]
focus_color = "Red"
break_color = "Green"
long_break_color = "Cyan"
digit_color = "White"

[ui]
ghost_mode = false
status_file = "/tmp/termato.status"   # 默认值；Windows 自动使用缓存目录

[hooks]
# 事件触发时执行的 Shell 命令。可用环境变量：
#   $TERMATO_EVENT (start|break|complete)
#   $TERMATO_TASK (任务名)
# on_start = "osascript -e 'set volume output muted true'"
# on_break = "osascript -e 'set volume output muted false'"
# on_complete = "afplay /System/Library/Sounds/Glass.aiff"
```

主题颜色支持颜色名称（`Red`、`Green`、`Cyan` 等）和十六进制值（`#ff5555`）。

## 数据存储

| 平台    | 配置文件 | 历史日志 | 状态文件 |
|---------|---------|---------|---------|
| macOS   | `~/Library/Application Support/termato/config.toml` | `~/.local/share/termato/history.log` | `~/Library/Caches/termato/termato.status` |
| Linux   | `~/.config/termato/config.toml` | `~/.local/share/termato/history.log` | `~/.cache/termato/termato.status` |
| Windows | `%APPDATA%\termato\config.toml` | `%LOCALAPPDATA%\termato\history.log` | `%LOCALAPPDATA%\termato\termato.status` |

## 许可证

MIT
