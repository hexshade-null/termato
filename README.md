# termato

A beautiful terminal Pomodoro timer built with Rust.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)

## Features

- **TUI Interface** — Large ASCII-art countdown, progress gauge, keyboard shortcuts
- **Pomodoro Cycle** — 25 min focus → 5 min short break → 15 min long break (every 4 rounds)
- **Task Tracking** — Associate a task name with each session (`-t "write docs #coding"`)
- **Configurable** — Customize durations, colors, and notifications via `~/.config/termato/config.toml`
- **Persistent Stats** — Session history logged to `~/.local/share/termato/history.log` (JSON Lines)
- **Desktop Notifications** — System-level alerts when a phase completes
- **Shell Completions** — Generate completion scripts for bash, zsh, fish, etc.

## Installation

### From source

```bash
git clone https://github.com/liujq/termato.git
cd termato
cargo build --release
cargo install --path .
```

### Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later

## Usage

```bash
termato                            # Start the TUI
termato start -t "写文档 #coding"    # Start with a task name
termato stats                      # Show today's focus stats
termato completion bash            # Generate shell completion script
```

### Keyboard Shortcuts

| Key     | Action          |
|---------|-----------------|
| Enter   | Start timer     |
| Space   | Pause / Resume  |
| R       | Reset phase     |
| S       | Skip phase      |
| Q / Esc | Quit            |

## Configuration

Copy the example config and edit to your liking:

```bash
mkdir -p ~/.config/termato
cp examples/config.toml ~/.config/termato/config.toml
```

Example `config.toml`:

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

Theme colors support named colors (`Red`, `Green`, `Cyan`, etc.) and hex values (`#ff5555`).

## Data Storage

- **Config**: `~/.config/termato/config.toml`
- **History**: `~/.local/share/termato/history.log` (JSON Lines format)

## License

MIT
