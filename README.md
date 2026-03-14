# Eye2020

A macOS menu bar utility that helps you follow the **20-20-20 rule** for eye strain prevention.

> Every **20 minutes** of screen time, look at something **20 feet** away for **20 seconds**.

## Features

- **Menu bar timer** — Live countdown in the macOS menu bar
- **Break reminders** — Animated countdown window with eye care tips
- **macOS notifications** — Native notification alerts when breaks are due
- **Reminder modes** — Window + Notification, Notification Only, Window Only, or Silent
- **Pause/Resume** — Quick toggle from the tray menu
- **Take Break Now** — Trigger a break anytime
- **Stays in tray** — Closing the window keeps the app running in the background

## Tech Stack

- **Rust** + **Tauri v2** (backend + native shell)
- **HTML/CSS/JS** (frontend webview)
- **Tokio** async runtime (battery-efficient timers)
- **tauri-plugin-notification** (macOS native notifications)

## Prerequisites

- **Rust** (install via [rustup](https://rustup.rs/))
- **Node.js** (for Tauri build tooling)
- **macOS** (primary target platform)

## Getting Started

```bash
# Navigate to the Rust project
cd EyeCare/src-tauri

# Build (debug)
cargo build

# Build (release — optimized, smaller binary)
cargo build --release

# Run (foreground — see logs in terminal)
cargo run

# Run release build (foreground)
cargo run --release
```

## Run in Background

```bash
# Start in background (release build)
cd EyeCare/src-tauri
cargo build --release
./target/release/eye2020 &

# Or using nohup (keeps running after terminal closes)
nohup ./target/release/eye2020 &

# Check if running
pgrep -x eye2020

# Stop the app
pkill -x eye2020

# Or stop by PID
pgrep -x eye2020    # get the PID
kill <PID>
```

## Quick Reference

| Action | Command |
|--------|---------|
| Build debug | `cargo build` |
| Build release | `cargo build --release` |
| Run foreground | `cargo run` |
| Run background | `./target/release/eye2020 &` |
| Run persistent | `nohup ./target/release/eye2020 &` |
| Check if running | `pgrep -x eye2020` |
| Stop | `pkill -x eye2020` |
| Quit from app | Tray menu → Quit Eye2020 |

The app will:
1. Open a dashboard window showing the 20-minute countdown timer
2. Place an icon in the macOS menu bar with live countdown
3. Send a notification + show a break overlay after 20 minutes
4. Closing the window hides it — the app keeps running in the tray

## Project Structure

```
src-tauri/
  src/
    main.rs                 — App entry point, Tauri setup
    commands.rs             — IPC commands (get_state, take_break, etc.)
    app/
      app_state.rs          — Shared state (timer, status, settings)
      timer_engine.rs       — Async countdown loop, triggers breaks
      break_manager.rs      — Break lifecycle + random eye care tips
    ui/
      tray.rs               — Menu bar tray icon and dropdown menu
    system/
      screen_monitor.rs     — macOS screen sleep/wake detection (WIP)
    data/
      stats_store.rs        — Statistics storage (WIP)
frontend/
  index.html                — Dashboard + break overlay UI
```

## Tray Menu

| Option | Action |
|--------|--------|
| Show Eye2020 | Open/focus the dashboard window |
| Take Break Now | Trigger an immediate eye break |
| Pause/Resume Monitoring | Toggle the countdown timer |
| Reminder mode options | Choose how breaks are delivered |
| Show Timer in Menu Bar | Toggle countdown display in tray |
| Quit Eye2020 | Exit the application |

## Roadmap

- [x] Core timer engine
- [x] Menu bar tray icon
- [x] Break reminder window with animated countdown
- [x] macOS notifications
- [x] Reminder mode selection
- [ ] Screen sleep/wake detection
- [ ] Statistics tracking (breaks completed, streaks)
- [ ] Settings persistence (SQLite)
- [ ] Auto-start on login
- [ ] Dark mode theme detection

## License

MIT
