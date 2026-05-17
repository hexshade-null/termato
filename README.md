# termato

A beautiful, cross-platform terminal Pomodoro timer built with Rust.

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-blue)

[中文文档](./README_zh.md)

## Features

### Core
- **TUI Interface** — Large ASCII-art countdown, Braille progress bar, keyboard shortcuts
- **Pomodoro Cycle** — 25 min focus → 5 min short break → 15 min long break (every 4 rounds)
- **Task Tracking** — Associate a task name with each session (`-t "write docs #coding"`)
- **Configurable** — Customize durations, colors, and notifications via config file
- **Persistent Stats** — Session history logged as JSON Lines
- **Desktop Notifications** — System-level alerts when a phase completes
- **Cross-Platform** — macOS, Linux, and Windows support

### Immersive & Anti-Distraction
- **Ghost Mode** — Hide countdown numbers, show only minimal progress bar (press `G` to toggle)
- **Anti-Accidental-Quit** — Confirmation dialog when quitting during active focus
- **Cursor Auto-Hide** — Terminal cursor hidden during TUI session

### Automation & Integration
- **Hook System** — Run shell commands on `on_start`, `on_break`, `on_complete` events
- **Status File Export** — Real-time state written to file for tmux/Polybar integration
- **Terminal Title Sync** — Dynamic window title shows current phase and task

### Smart Context
- **Git Auto-Detection** — Auto-fills task as `"repo: branch"` when launched inside a Git repo
- **Task Queue** — `termato add "Task A"` to queue tasks, auto-advance in TUI

### Visualization
- **Braille Progress Bar** — Unicode dot-matrix rendering for smooth progress display
- **Rest Animation** — Breathing circle animation during break periods
- **Heatmap Stats** — `termato stats --heatmap` shows GitHub-style yearly focus heatmap

## Installation

### From source

```bash
git clone https://github.com/hexshade-null/termato.git
cd termato
cargo build --release
cargo install --path .
```

### Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later

## Usage

```bash
termato                              # Start the TUI
termato start -t "Write docs"        # Start with a task name
termato add "Task A"                 # Add task to queue
termato add "Task B"                 # Queue multiple tasks
termato stats                        # Show today's focus stats
termato stats --heatmap              # Show yearly focus heatmap
termato completion bash              # Generate shell completion script
```

### Keyboard Shortcuts

| Key     | Action            |
|---------|-------------------|
| Enter   | Start timer       |
| Space   | Pause / Resume    |
| R       | Reset phase       |
| S       | Skip phase        |
| G       | Toggle Ghost Mode |
| Q / Esc | Quit              |

## Configuration

Copy the example config and edit to your liking:

```bash
# macOS / Linux
mkdir -p ~/.config/termato
cp examples/config.toml ~/.config/termato/config.toml

# Windows (PowerShell)
mkdir "$env:APPDATA\termato"
Copy-Item examples/config.toml "$env:APPDATA\termato\config.toml"
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

[ui]
ghost_mode = false
status_file = "/tmp/termato.status"   # default; Windows uses cache dir

[hooks]
# Run shell commands on events. Environment variables available:
#   $TERMATO_EVENT (start|break|complete)
#   $TERMATO_TASK (task name)
# on_start = "osascript -e 'set volume output muted true'"
# on_break = "osascript -e 'set volume output muted false'"
# on_complete = "afplay /System/Library/Sounds/Glass.aiff"
```

Theme colors support named colors (`Red`, `Green`, `Cyan`, etc.) and hex values (`#ff5555`).

## Data Storage

| Platform | Config | History | Status File |
|----------|--------|---------|-------------|
| macOS    | `~/Library/Application Support/termato/config.toml` | `~/.local/share/termato/history.log` | `~/Library/Caches/termato/termato.status` |
| Linux    | `~/.config/termato/config.toml` | `~/.local/share/termato/history.log` | `~/.cache/termato/termato.status` |
| Windows  | `%APPDATA%\termato\config.toml` | `%LOCALAPPDATA%\termato\history.log` | `%LOCALAPPDATA%\termato\termato.status` |

## License

MIT
