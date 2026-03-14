use super::app_state::{MonitoringStatus, ReminderMode, SharedAppState};
use super::break_manager;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

/// Runs the main timer loop. Ticks every second when active,
/// counts down to zero, then triggers the break reminder.
pub async fn run_timer(app: AppHandle, state: SharedAppState) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut s = state.lock().await;

        match s.status {
            MonitoringStatus::Active => {
                if s.seconds_remaining > 0 {
                    s.seconds_remaining -= 1;
                }

                if s.seconds_remaining == 0 {
                    s.status = MonitoringStatus::OnBreak;
                    let reminder_mode = s.reminder_mode;
                    drop(s);

                    trigger_break(&app, reminder_mode);
                    continue;
                }

                let remaining = s.seconds_remaining;
                let show_in_menu = s.show_timer_in_menu;
                drop(s);

                // Update tray icon title with countdown
                if show_in_menu {
                    let mins = remaining / 60;
                    let secs = remaining % 60;
                    update_tray_title(&app, &format!("{:02}:{:02}", mins, secs));
                }
            }
            MonitoringStatus::Paused => {
                drop(s);
                update_tray_title(&app, "⏸");
            }
            MonitoringStatus::OnBreak => {
                // Do nothing, waiting for user to finish break
            }
        }
    }
}

fn trigger_break(app: &AppHandle, mode: ReminderMode) {
    let tip = break_manager::random_tip();

    // Send notification if enabled
    match mode {
        ReminderMode::WindowAndNotification | ReminderMode::NotificationOnly => {
            let _ = app
                .notification()
                .builder()
                .title("Eye2020 — Time for a Break!")
                .body(tip)
                .show();
        }
        _ => {}
    }

    // Show break overlay in main window (no separate window)
    match mode {
        ReminderMode::WindowAndNotification | ReminderMode::WindowOnly => {
            // Show and focus the main window
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
            let _ = app.emit("break-due", ());
        }
        _ => {
            let _ = app.emit("break-due", ());
        }
    }
}

fn update_tray_title(app: &AppHandle, title: &str) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_title(Some(title));
    }
}

/// Sends a persistent-style status notification every 5 minutes
/// showing the remaining time. Clicking it opens the app.
pub async fn run_status_notifications(app: AppHandle, state: SharedAppState) {
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await; // every 5 min

        let s = state.lock().await;
        if s.status != MonitoringStatus::Active {
            continue;
        }
        let remaining = s.seconds_remaining;
        drop(s);

        let mins = remaining / 60;
        let body = if mins > 0 {
            format!("Next eye break in {} min. Click to open Eye2020.", mins)
        } else {
            "Eye break coming up soon!".to_string()
        };

        let _ = app
            .notification()
            .builder()
            .title("👁 Eye2020 — Monitoring")
            .body(&body)
            .show();
    }
}
