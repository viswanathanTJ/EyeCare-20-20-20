#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod data;
mod system;
mod ui;

use app::app_state;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process;
use tauri::Manager;

fn socket_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(".eye2020.sock");
    path
}

fn lock_file_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(".eye2020.lock");
    path
}

/// Try to connect to an existing instance via Unix socket.
/// If successful, send "show" command and exit.
/// If not, we are the first instance — start normally.
fn check_single_instance() {
    let sock = socket_path();
    let lock_path = lock_file_path();

    // Try connecting to existing instance
    if sock.exists() {
        if let Ok(mut stream) = UnixStream::connect(&sock) {
            let _ = stream.write_all(b"show");
            eprintln!("Eye2020 is already running — bringing window to front.");
            process::exit(0);
        }
        // Socket exists but no one is listening — stale, clean up
        let _ = fs::remove_file(&sock);
    }

    // Also clean up stale lock file
    if lock_path.exists() {
        if let Ok(mut file) = fs::File::open(&lock_path) {
            let mut pid_str = String::new();
            if file.read_to_string(&mut pid_str).is_ok() {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    let status = process::Command::new("kill")
                        .args(["-0", &pid.to_string()])
                        .output();
                    if let Ok(output) = status {
                        if output.status.success() {
                            // Process alive but socket gone — try sending SIGUSR1 as fallback
                            eprintln!("Eye2020 is already running (PID {}). Attempting to show window...", pid);
                            let _ = process::Command::new("kill")
                                .args(["-USR1", &pid.to_string()])
                                .output();
                            process::exit(0);
                        }
                    }
                }
            }
        }
        let _ = fs::remove_file(&lock_path);
    }

    // Write lock file with our PID
    if let Ok(mut file) = fs::File::create(&lock_path) {
        let _ = file.write_all(process::id().to_string().as_bytes());
    }
}

/// Start a Unix socket listener that waits for "show" messages
/// from subsequent CLI invocations and shows the main window.
fn start_ipc_listener(app_handle: tauri::AppHandle) {
    let sock = socket_path();
    let _ = fs::remove_file(&sock); // clean before bind

    let listener = match UnixListener::bind(&sock) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind IPC socket: {}", e);
            return;
        }
    };

    // Non-blocking accept in a background thread
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buf = [0u8; 16];
                if let Ok(n) = stream.read(&mut buf) {
                    let msg = String::from_utf8_lossy(&buf[..n]);
                    if msg.starts_with("show") {
                        if let Some(win) = app_handle.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                }
            }
        }
    });
}

fn cleanup_lock() {
    let _ = fs::remove_file(lock_file_path());
    let _ = fs::remove_file(socket_path());
}

fn main() {
    check_single_instance();

    let state = app_state::new_shared_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_tip,
            commands::finish_break,
            commands::skip_break,
            commands::pause_monitoring,
            commands::resume_monitoring,
            commands::take_break_now,
            commands::set_reminder_mode,
            commands::toggle_timer_in_menu,
            commands::set_interval,
            commands::set_break_duration,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap_or_default();
                api.prevent_close();
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            let state_clone = state.clone();

            // Create main window
            let _main_window = tauri::WebviewWindowBuilder::new(
                &handle,
                "main",
                tauri::WebviewUrl::App("/".into()),
            )
            .title("Eye2020")
            .inner_size(440.0, 750.0)
            .center()
            .build()?;

            // Setup tray icon and menu
            ui::tray::setup_tray(&handle, state_clone.clone())?;

            // Setup screen monitoring (placeholder)
            system::screen_monitor::setup_screen_monitoring(&handle, state_clone.clone());

            // Start the timer engine
            let timer_handle = handle.clone();
            let timer_state = state_clone.clone();
            tauri::async_runtime::spawn(async move {
                app::timer_engine::run_timer(timer_handle, timer_state).await;
            });

            // Start periodic status notification (every 5 minutes)
            let notif_handle = handle.clone();
            let notif_state = state_clone.clone();
            tauri::async_runtime::spawn(async move {
                app::timer_engine::run_status_notifications(notif_handle, notif_state).await;
            });

            // Start IPC listener for single-instance "show" commands from CLI
            start_ipc_listener(handle.clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building Eye2020")
        .run(|app, event| {
            match event {
                tauri::RunEvent::Reopen { has_visible_windows, .. } => {
                    // Dock icon clicked — show window
                    if !has_visible_windows {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                }
                tauri::RunEvent::Exit => {
                    cleanup_lock();
                }
                _ => {}
            }
        });
}
