use crate::app::app_state::{ReminderMode, SharedAppState};
use crate::app::break_manager;
use crate::data::stats_store::StatsStore;
use tauri::State;

#[tauri::command]
pub async fn get_state(state: State<'_, SharedAppState>) -> Result<crate::app::app_state::AppState, String> {
    let s = state.lock().await;
    Ok(s.clone())
}

#[tauri::command]
pub async fn get_tip() -> Result<String, String> {
    Ok(break_manager::random_tip().to_string())
}

#[tauri::command]
pub async fn finish_break(state: State<'_, SharedAppState>) -> Result<(), String> {
    break_manager::finish_break(&state).await;
    Ok(())
}

#[tauri::command]
pub async fn skip_break(state: State<'_, SharedAppState>) -> Result<(), String> {
    break_manager::skip_break(&state).await;
    Ok(())
}

#[tauri::command]
pub async fn pause_monitoring(state: State<'_, SharedAppState>) -> Result<(), String> {
    break_manager::pause_monitoring(&state).await;
    Ok(())
}

#[tauri::command]
pub async fn resume_monitoring(state: State<'_, SharedAppState>) -> Result<(), String> {
    break_manager::resume_monitoring(&state).await;
    Ok(())
}

#[tauri::command]
pub async fn take_break_now(state: State<'_, SharedAppState>) -> Result<(), String> {
    break_manager::take_break_now(&state).await;
    Ok(())
}

#[tauri::command]
pub async fn set_reminder_mode(state: State<'_, SharedAppState>, mode: ReminderMode) -> Result<(), String> {
    let mut s = state.lock().await;
    s.reminder_mode = mode;
    // Persist
    if let Ok(store) = StatsStore::open() {
        let mode_str = serde_json::to_string(&mode).unwrap_or_default();
        let _ = store.save_setting("reminder_mode", &mode_str);
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_timer_in_menu(state: State<'_, SharedAppState>) -> Result<bool, String> {
    let mut s = state.lock().await;
    s.show_timer_in_menu = !s.show_timer_in_menu;
    if let Ok(store) = StatsStore::open() {
        let _ = store.save_setting("show_timer_in_menu", &s.show_timer_in_menu.to_string());
    }
    Ok(s.show_timer_in_menu)
}

#[tauri::command]
pub async fn set_interval(state: State<'_, SharedAppState>, minutes: u64) -> Result<(), String> {
    if minutes < 1 || minutes > 120 {
        return Err("Interval must be between 1 and 120 minutes".into());
    }
    let mut s = state.lock().await;
    s.break_interval_secs = minutes * 60;
    s.seconds_remaining = minutes * 60;
    if let Ok(store) = StatsStore::open() {
        let _ = store.save_setting("break_interval_secs", &(minutes * 60).to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn set_break_duration(state: State<'_, SharedAppState>, seconds: u64) -> Result<(), String> {
    if seconds < 5 || seconds > 300 {
        return Err("Break duration must be between 5 and 300 seconds".into());
    }
    let mut s = state.lock().await;
    s.break_duration_secs = seconds;
    if let Ok(store) = StatsStore::open() {
        let _ = store.save_setting("break_duration_secs", &seconds.to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn get_stats() -> Result<serde_json::Value, String> {
    let store = StatsStore::open()?;
    let (completed, skipped) = store.get_today_stats();
    let (current_streak, longest_streak) = store.get_streak();
    let total = store.get_total_breaks();

    Ok(serde_json::json!({
        "today_completed": completed,
        "today_skipped": skipped,
        "current_streak": current_streak,
        "longest_streak": longest_streak,
        "total_breaks": total,
    }))
}

#[tauri::command]
pub async fn toggle_sound(state: State<'_, SharedAppState>) -> Result<bool, String> {
    let mut s = state.lock().await;
    s.sound_enabled = !s.sound_enabled;
    if let Ok(store) = StatsStore::open() {
        let _ = store.save_setting("sound_enabled", &s.sound_enabled.to_string());
    }
    Ok(s.sound_enabled)
}

#[tauri::command]
pub async fn set_auto_start(state: State<'_, SharedAppState>, enabled: bool) -> Result<(), String> {
    let mut s = state.lock().await;
    s.auto_start = enabled;

    if let Ok(store) = StatsStore::open() {
        let _ = store.save_setting("auto_start", &enabled.to_string());
    }

    // macOS: create/remove LaunchAgent plist
    if enabled {
        create_launch_agent()?;
    } else {
        remove_launch_agent()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_theme() -> Result<String, String> {
    Ok(detect_macos_theme())
}

/// Detect macOS dark/light mode
pub fn detect_macos_theme() -> String {
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.trim().eq_ignore_ascii_case("dark") {
                "dark".to_string()
            } else {
                "light".to_string()
            }
        }
        Err(_) => "light".to_string(), // default to light if can't detect
    }
}

/// Load persisted settings from SQLite into the app state
pub async fn load_persisted_settings(state: &SharedAppState) {
    if let Ok(store) = StatsStore::open() {
        let mut s = state.lock().await;

        if let Some(val) = store.load_setting("break_interval_secs") {
            if let Ok(n) = val.parse::<u64>() {
                s.break_interval_secs = n;
                s.seconds_remaining = n;
            }
        }
        if let Some(val) = store.load_setting("break_duration_secs") {
            if let Ok(n) = val.parse::<u64>() {
                s.break_duration_secs = n;
            }
        }
        if let Some(val) = store.load_setting("reminder_mode") {
            if let Ok(mode) = serde_json::from_str::<ReminderMode>(&val) {
                s.reminder_mode = mode;
            }
        }
        if let Some(val) = store.load_setting("show_timer_in_menu") {
            if let Ok(b) = val.parse::<bool>() {
                s.show_timer_in_menu = b;
            }
        }
        if let Some(val) = store.load_setting("sound_enabled") {
            if let Ok(b) = val.parse::<bool>() {
                s.sound_enabled = b;
            }
        }
        if let Some(val) = store.load_setting("auto_start") {
            if let Ok(b) = val.parse::<bool>() {
                s.auto_start = b;
            }
        }

        // Load stats
        let (completed, skipped) = store.get_today_stats();
        let (current, longest) = store.get_streak();
        s.today_completed = completed;
        s.today_skipped = skipped;
        s.current_streak = current;
        s.longest_streak = longest;
        s.total_breaks = store.get_total_breaks();

        // Detect theme
        s.theme = detect_macos_theme();
    }
}

fn launch_agent_path() -> std::path::PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    path.push("Library/LaunchAgents/com.eye2020.app.plist");
    path
}

fn create_launch_agent() -> Result<(), String> {
    let app_path = "/Applications/Eye2020.app/Contents/MacOS/Eye2020";
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.eye2020.app</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>"#,
        app_path
    );

    let path = launch_agent_path();
    std::fs::write(&path, plist).map_err(|e| format!("Failed to create LaunchAgent: {}", e))?;

    // Load it immediately
    let _ = std::process::Command::new("launchctl")
        .args(["load", path.to_str().unwrap_or_default()])
        .output();

    Ok(())
}

fn remove_launch_agent() -> Result<(), String> {
    let path = launch_agent_path();
    if path.exists() {
        // Unload first
        let _ = std::process::Command::new("launchctl")
            .args(["unload", path.to_str().unwrap_or_default()])
            .output();
        std::fs::remove_file(&path).map_err(|e| format!("Failed to remove LaunchAgent: {}", e))?;
    }
    Ok(())
}
