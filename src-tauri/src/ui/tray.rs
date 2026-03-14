use crate::app::app_state::{MonitoringStatus, ReminderMode, SharedAppState};
use crate::app::break_manager;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

pub fn setup_tray(app: &AppHandle, state: SharedAppState) -> tauri::Result<()> {
    let show_window = MenuItem::with_id(app, "show_window", "Show Eye2020", true, None::<&str>)?;
    let sep0 = PredefinedMenuItem::separator(app)?;
    let take_break = MenuItem::with_id(app, "take_break", "Take Break Now", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause Monitoring", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;

    let mode_window_notif = MenuItem::with_id(app, "mode_window_notif", "✓ Window + Notification", true, None::<&str>)?;
    let mode_notif_only = MenuItem::with_id(app, "mode_notif_only", "  Notification Only", true, None::<&str>)?;
    let mode_window_only = MenuItem::with_id(app, "mode_window_only", "  Window Only", true, None::<&str>)?;
    let mode_silent = MenuItem::with_id(app, "mode_silent", "  Silent", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;

    let toggle_timer = MenuItem::with_id(app, "toggle_timer", "✓ Show Timer in Menu Bar", true, None::<&str>)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Eye2020", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_window,
            &sep0,
            &take_break,
            &pause,
            &sep1,
            &mode_window_notif,
            &mode_notif_only,
            &mode_window_only,
            &mode_silent,
            &sep2,
            &toggle_timer,
            &sep3,
            &quit,
        ],
    )?;

    let state_clone = state.clone();

    TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .menu(&menu)
        .title("20:00")
        .tooltip("Eye2020 — 20-20-20 Rule")
        .on_menu_event(move |app, event| {
            let state = state_clone.clone();
            let app = app.clone();

            tauri::async_runtime::spawn(async move {
                match event.id.as_ref() {
                    "show_window" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "take_break" => {
                        break_manager::take_break_now(&state).await;
                        let _ = app.emit("break-due", ());
                    }
                    "pause" => {
                        let mut s = state.lock().await;
                        match s.status {
                            MonitoringStatus::Active => {
                                s.status = MonitoringStatus::Paused;
                            }
                            MonitoringStatus::Paused => {
                                s.status = MonitoringStatus::Active;
                            }
                            _ => {}
                        }
                    }
                    "mode_window_notif" => {
                        let mut s = state.lock().await;
                        s.reminder_mode = ReminderMode::WindowAndNotification;
                    }
                    "mode_notif_only" => {
                        let mut s = state.lock().await;
                        s.reminder_mode = ReminderMode::NotificationOnly;
                    }
                    "mode_window_only" => {
                        let mut s = state.lock().await;
                        s.reminder_mode = ReminderMode::WindowOnly;
                    }
                    "mode_silent" => {
                        let mut s = state.lock().await;
                        s.reminder_mode = ReminderMode::Silent;
                    }
                    "toggle_timer" => {
                        let mut s = state.lock().await;
                        s.show_timer_in_menu = !s.show_timer_in_menu;
                        if !s.show_timer_in_menu {
                            if let Some(tray) = app.tray_by_id("main-tray") {
                                let _ = tray.set_title(Some(""));
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            });
        })
        .build(app)?;

    Ok(())
}
