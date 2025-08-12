# Pomodoros — Rust TUI Pomodoro Timer

A modern, lightweight Pomodoro timer for your terminal. Built with Rust + ratatui with a smooth TUI, clear progress, and keyboard-first controls to help you focus and rest effectively.

---

## Features
- Modern TUI: rounded borders, clear panels, progress with inline remaining time
- Keyboard-first: start/pause, skip, reset, quit
- Highly configurable: focus/short/long durations and long-break interval
- Auto switching: Focus → Break (long break after every N focus sessions)
- Audible bell on phase completion (toggle with `--mute`)
- Cross-platform: macOS, Linux, Windows (Windows Terminal)

---

## Installation

### Homebrew (recommended, custom tap)
If you maintain a tap (e.g., `zh30/homebrew-tap`):
```bash
brew tap zh30/tap
brew install pomodoros
```

### Cargo (from source)
Requires Rust toolchain (`https://www.rust-lang.org/`).
```bash
# From repository root (during development)
cargo install --path .

# After publishing, from Git
cargo install --git https://github.com/zh30/pomodoros
```

---

## Usage
```bash
pomodoros
pomodoros --help
```
Common options:
```
Rust TUI Pomodoro Timer

Options:
  -f, --focus <MIN>     Focus duration in minutes (default: 25)
  -s, --short <MIN>     Short break in minutes (default: 5)
  -l, --long <MIN>      Long break in minutes (default: 15)
  -e, --every <N>       Take a long break after every N focus sessions (default: 4)
      --mute            Mute terminal bell
      --tick <MS>       Tick interval in milliseconds (default: 200)
  -h, --help            Print help
  -V, --version         Print version
```

Examples:
```bash
# 50-minute focus, 10-minute short break, 20-minute long break, long break every 3 sessions
pomodoros -f 50 -s 10 -l 20 -e 3

# Mute and set faster tick
pomodoros --mute --tick 100
```

### Shortcuts
- Space: Start / Pause
- n or →: Skip current phase
- r: Reset current phase
- q / Esc / Ctrl+C: Quit

---

## Screenshot (mock)
```
┌──────────────────────── Status ────────────────────────┐
│ ● Focus  ·  Completed 3                                │
└────────────────────────────────────────────────────────┘
┌────────────────────── Progress ────────────────────────┐
│███████████████████▌  13:42  ·  54%                     │
└────────────────────────────────────────────────────────┘
┌──────────────────────── Timer ─────────────────────────┐
│                      13:42                             │
│                      ⏱ Running                         │
└────────────────────────────────────────────────────────┘
┌────────────────────── Shortcuts ───────────────────────┐
│ ␣ Space: Start/Pause  ·  ⏭ n: Skip  ·  ⟲ r: Reset  ·  q: Quit │
└────────────────────────────────────────────────────────┘
```

---

## Development
```bash
cargo build
cargo run
cargo build --release
```
Entry point: `src/main.rs`.

---

## CI & Homebrew (brief)
- Keep a dedicated tap repo: `zh30/homebrew-tap` with formula `Formula/pomodoros.rb`.
- Tag (e.g., `v0.1.0`) triggers GitHub Actions:
  - Build and create GitHub Release
  - Update the tap formula to point to the tagged source tarball
- Users install via: `brew tap zh30/tap && brew install pomodoros`

Optional tools: `mislav/bump-homebrew-formula-action`, `softprops/action-gh-release`, `cargo-dist` (for bottles if needed).

---

## Contributing
PRs and issues are welcome. Please keep code readable and follow Clippy/lints.

---

## License
MIT (see `LICENSE`).
