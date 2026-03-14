use crate::app::app_state::{MonitoringStatus, SharedAppState};
use std::time::Duration;
use tauri::AppHandle;

/// Monitors screen lock/unlock state.
/// On lock → pauses timer.
/// On unlock → resets timer to full interval and resumes.
pub fn setup_screen_monitoring(_app: &AppHandle, state: SharedAppState) {
    std::thread::spawn(move || {
        let mut was_locked = false;

        loop {
            std::thread::sleep(Duration::from_secs(2));

            let locked = is_session_locked();

            if locked && !was_locked {
                                let st = state.clone();
                tauri::async_runtime::block_on(async {
                    let mut s = st.lock().await;
                    if s.status == MonitoringStatus::Active {
                        s.status = MonitoringStatus::Paused;
                    }
                });
            } else if !locked && was_locked {
                                let st = state.clone();
                tauri::async_runtime::block_on(async {
                    let mut s = st.lock().await;
                    if s.status == MonitoringStatus::Paused {
                        s.seconds_remaining = s.break_interval_secs;
                        s.status = MonitoringStatus::Active;
                    }
                });
            }

            was_locked = locked;
        }
    });
}

#[cfg(target_os = "macos")]
fn is_session_locked() -> bool {
    use std::process::Command;

    let output = Command::new("python3")
        .args(["-c", r#"
import subprocess, plistlib
r = subprocess.run(['ioreg', '-n', 'Root', '-d1', '-a'], capture_output=True)
d = plistlib.loads(r.stdout)
users = d.get('IOConsoleUsers', [])
for u in users:
    if u.get('kCGSSessionOnConsoleKey', False):
        print('1' if u.get('CGSSessionScreenIsLocked', False) else '0')
        break
else:
    print('0')
"#])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let result = stdout.trim() == "1";
            result
        }
        Err(_) => {
            false
        }
    }
}

#[cfg(target_os = "windows")]
fn is_session_locked() -> bool {
    false
}

#[cfg(target_os = "linux")]
fn is_session_locked() -> bool {
    use std::process::Command;
    let output = Command::new("loginctl")
        .args(["show-session", "self", "-p", "LockedHint"])
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains("yes")
        }
        Err(_) => false,
    }
}
