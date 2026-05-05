# termato

一个基于 Rust 构建的终端番茄钟工具。

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)

[English](./README.md)

## 功能特性

- **TUI 界面** — 大号 ASCII 倒计时数字、进度条、快捷键提示
- **番茄工作法循环** — 25 分钟专注 → 5 分钟短休息 → 15 分钟长休息（每 4 轮）
- **任务追踪** — 为每次会话关联任务名称（`-t "写文档 #coding"`）
- **可配置** — 通过 `~/.config/termato/config.toml` 自定义时长、配色、通知等
- **数据持久化** — 会话记录以 JSON Lines 格式写入 `~/.local/share/termato/history.log`
- **桌面通知** — 阶段结束时发送系统级通知
- **Shell 补全** — 支持生成 bash、zsh、fish 等补全脚本

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
termato                            # 启动 TUI
termato start -t "写文档 #coding"    # 带任务名启动
termato stats                      # 查看今日专注统计
termato completion bash            # 生成 Shell 补全脚本
```

### 快捷键

| 按键    | 操作       |
|---------|-----------|
| Enter   | 开始计时   |
| Space   | 暂停 / 继续 |
| R       | 重置当前阶段 |
| S       | 跳过当前阶段 |
| Q / Esc | 退出       |

## 配置

复制示例配置并按需修改：

```bash
mkdir -p ~/.config/termato
cp examples/config.toml ~/.config/termato/config.toml
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
```

主题颜色支持颜色名称（`Red`、`Green`、`Cyan` 等）和十六进制值（`#ff5555`）。

## 数据存储

- **配置文件**：`~/.config/termato/config.toml`
- **历史日志**：`~/.local/share/termato/history.log`（JSON Lines 格式）

## 许可证

MIT
