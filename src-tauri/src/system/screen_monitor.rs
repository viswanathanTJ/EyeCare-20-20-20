use crate::app::app_state::{MonitoringStatus, SharedAppState};
use std::process::Command;
use std::time::Duration;
use tauri::AppHandle;

/// Monitors macOS screen sleep/wake state by polling IOKit.
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
                        // We'll track that we auto-paused so we auto-resume
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

/// Check if the display is asleep using IOKit via ioreg.
fn is_screen_asleep() -> bool {
    let output = Command::new("ioreg")
        .args(["-rc", "AppleBacklightDisplay"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // If no display info returned, screen is likely asleep
            if stdout.trim().is_empty() {
                return false; // can't determine, assume awake
            }
            // Check for brightness = 0 or powered off indicators
            // A simpler check: use pmset
            is_display_off_pmset()
        }
        Err(_) => false,
    }
}

/// Use `pmset -g powerstate` to check display power state.
fn is_display_off_pmset() -> bool {
    let output = Command::new("pmset")
        .args(["-g", "powerstate", "IODisplayWrangler"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Power state 1 = off/sleep, 4 = on
            for line in stdout.lines() {
                if line.contains("IODisplayWrangler") {
                    // The last column is the current power state
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
