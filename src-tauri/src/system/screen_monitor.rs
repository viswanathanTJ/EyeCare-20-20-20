use crate::app::app_state::{MonitoringStatus, SharedAppState};
use std::time::Duration;
use tauri::AppHandle;

/// Monitors screen sleep/wake state by polling platform-specific APIs.
/// On screen sleep → pauses timer automatically.
/// On screen wake → resumes timer automatically.
pub fn setup_screen_monitoring(_app: &AppHandle, state: SharedAppState) {
    std::thread::spawn(move || {
        let mut was_asleep = false;
        let mut auto_paused = false;

        loop {
            std::thread::sleep(Duration::from_secs(5));

            let asleep = is_screen_asleep();

            if asleep && !was_asleep {
                // Screen just went to sleep — auto-pause
                let st = state.clone();
                tauri::async_runtime::block_on(async {
                    let mut s = st.lock().await;
                    if s.status == MonitoringStatus::Active {
                        s.status = MonitoringStatus::Paused;
                    }
                });
                auto_paused = true;
            } else if !asleep && was_asleep && auto_paused {
                // Screen just woke up — auto-resume if we auto-paused
                let st = state.clone();
                tauri::async_runtime::block_on(async {
                    let mut s = st.lock().await;
                    if s.status == MonitoringStatus::Paused {
                        s.status = MonitoringStatus::Active;
                    }
                });
                auto_paused = false;
            }

            was_asleep = asleep;
        }
    });
}

/// Platform-specific screen sleep detection.

#[cfg(target_os = "macos")]
fn is_screen_asleep() -> bool {
    use std::process::Command;

    let output = Command::new("ioreg")
        .args(["-rc", "AppleBacklightDisplay"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.trim().is_empty() {
                return false;
            }
            is_display_off_pmset()
        }
        Err(_) => false,
    }
}

#[cfg(target_os = "macos")]
fn is_display_off_pmset() -> bool {
    use std::process::Command;

    let output = Command::new("pmset")
        .args(["-g", "powerstate", "IODisplayWrangler"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("IODisplayWrangler") {
                    if let Some(state) = line.split_whitespace().last() {
                        if let Ok(n) = state.parse::<u32>() {
                            return n < 4;
                        }
                    }
                }
            }
            false
        }
        Err(_) => false,
    }
}

#[cfg(target_os = "windows")]
fn is_screen_asleep() -> bool {
    // Windows doesn't expose a simple CLI for display power state.
    // Always report awake — screen monitoring is best-effort.
    false
}

#[cfg(target_os = "linux")]
fn is_screen_asleep() -> bool {
    use std::process::Command;

    // Try xset for X11
    let output = Command::new("xset")
        .args(["-q"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Look for "Monitor is Off"
            stdout.contains("Monitor is Off")
        }
        Err(_) => false,
    }
}
