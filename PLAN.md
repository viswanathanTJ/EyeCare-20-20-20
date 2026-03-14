# PLAN.md

## Role

You are a **senior macOS systems engineer and Rust architect**.
Your task is to design a **production-quality macOS application written in Rust** that enforces the **20-20-20 eye rule**.

You must:

* design the full architecture
* explain system components
* ensure macOS best practices
* prioritize battery efficiency
* use modern Rust ecosystem tools
* provide a clear implementation roadmap

Do **not immediately generate large code blocks**.
First design the **architecture and engineering plan**.

---

# Application Goal

Build a **macOS background utility** that helps users follow the **20-20-20 rule**:

Every **20 minutes of continuous screen usage**, the user must look **20 feet away for 20 seconds**.

The application should:

* run quietly in the background
* be extremely lightweight
* be pleasant and motivating to use
* provide reminders and visual breaks

---

# Application Type

The application must be implemented as a **macOS menu bar utility**.

The app should:

* run continuously
* show an icon in the macOS menu bar
* run with minimal CPU usage
* allow quick enable/disable
* display break reminders

The menu bar is the primary interface.

---

# Core Behavior

The application monitors screen activity.

### Timer Logic

1. When the screen becomes active:

   * start a **20 minute timer**

2. If the screen sleeps or the computer locks:

   * pause or reset the timer

3. After **20 minutes of continuous screen activity**:

   * trigger a **break reminder**

---

# Break Reminder

The reminder must trigger a **20-second eye break session**.

The break window should display:

* animated countdown
* relaxing visual design
* eye care tips
* eye health quotes

Example messages:

* "Look at something 20 feet away."
* "Relax your eye muscles."
* "Blink slowly for a few seconds."

---

# Break Screen Behavior

Break window must include:

* 20 second animated countdown
* relaxing UI
* progress circle animation
* motivational message

User actions:

* Start break
* Skip break
* Dismiss reminder

---

# Menu Bar Interface

The menu bar dropdown should show:

App Name: Eye2020

Menu Example:

Status: Monitoring
Next break: 12:14

Menu options:

Take Break Now
Pause Monitoring
Resume Monitoring
Statistics
Settings
Quit

---

# Settings

The settings window must allow configuration of:

### Reminder Behavior

Reminder modes:

1. Full break window
2. Notification reminder
3. Silent background reminder
4. Overlay reminder

### Notification Options

* Enable sound
* Enable vibration / haptic (if supported)

### Timer Behavior

* Enable or disable reminders
* Pause monitoring

### Startup

* Launch app automatically at login

---

# Statistics

The application should track:

Daily stats:

* number of breaks completed
* number of breaks skipped
* total monitoring sessions

Gamification:

### Streak System

Track consecutive days following the rule.

Examples:

3 day streak
7 day streak
30 day streak

Statistics should be visible in a **statistics window**.

---

# Quotes and Eye Tips

During breaks the app should display rotating quotes.

Examples:

"Blink often to keep eyes hydrated."

"Look far away to relax eye muscles."

"Short breaks protect long-term vision."

Quotes should rotate randomly.

---

# Dark Mode

The application must support:

* full dark mode
* automatic macOS theme detection

Color palette must be **eye-friendly**.

Avoid bright colors and strong contrast.

---

# System Integration

The application must integrate properly with macOS.

### Detect Screen Sleep / Wake

Use macOS system notifications such as:

* screen sleep
* screen wake
* session lock
* session unlock

Use macOS APIs through Rust bindings.

---

# Auto Start on Login

The application must support **auto start when the user logs in**.

Implementation options:

* LaunchAgent
* SMLoginItemSetEnabled
* login item registration

The user must be able to enable or disable this in settings.

---

# Rust Technology Stack

The application must use Rust.

Suggested stack:

Core runtime:

* Rust
* Tokio async runtime

Menu bar:

* tray-icon crate

Notifications:

* mac-notification-sys

macOS integration:

* cocoa
* objc
* core-foundation

UI framework options (choose the best approach):

Option A (recommended):

* Tauri + Rust backend

Option B:

* egui / eframe (pure Rust UI)

---

# Battery Efficiency Requirements

The application must be highly efficient.

Requirements:

* no tight loops
* use async timers
* avoid constant polling
* rely on macOS system events
* minimal CPU usage

The timer must sleep efficiently between checks.

---

# Data Storage

Use local storage for statistics and settings.

Recommended solutions:

* SQLite
* sled
* or lightweight JSON config

Data to store:

settings
daily stats
streak history

---

# Project Structure

Define a clean Rust project architecture.

Example:

src/

main.rs

app/
app_state.rs
timer_engine.rs
break_manager.rs

ui/
tray_menu.rs
break_window.rs
settings_window.rs
stats_window.rs

system/
screen_monitor.rs
autostart.rs
notifications.rs

data/
config_store.rs
stats_store.rs

---

# Required Deliverables

Generate a full engineering design including:

1. System architecture
2. Core modules and responsibilities
3. Timer engine design
4. Screen activity detection
5. Break reminder flow
6. Menu bar architecture
7. Settings system
8. Statistics tracking design
9. Data storage design
10. Battery optimization strategy
11. UI structure
12. Implementation roadmap

---

# Development Phases

After the architecture is defined, generate a step-by-step development plan.

Example phases:

Phase 1
Core timer engine

Phase 2
Menu bar UI

Phase 3
Break reminder window

Phase 4
Statistics system

Phase 5
Settings and configuration

Phase 6
macOS integration

Phase 7
Optimization and polish

---

# Output Format

Produce the output as a **professional engineering design document**.

Structure:

Architecture
Modules
System Flow
Implementation Plan
Development Phases

The document must be **clear enough for a developer to immediately start building the application**.
