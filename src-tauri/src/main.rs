#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod data;
mod system;
mod ui;

use app::app_state;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process;
use tauri::Manager;

fn lock_file_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from(default_tmp()));
    path.push(".eye2020.lock");
    path
}

fn default_tmp() -> &'static str {
    #[cfg(unix)]
    { "/tmp" }
    #[cfg(windows)]
    { "C:\\Temp" }
}

// ── Unix single-instance IPC via Unix socket ──

#[cfg(unix)]
fn socket_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(".eye2020.sock");
    path
}

#[cfg(unix)]
fn check_single_instance() {
    use std::os::unix::net::UnixStream;

    let sock = socket_path();
    let lock_path = lock_file_path();

    if sock.exists() {
        if let Ok(mut stream) = UnixStream::connect(&sock) {
            let _ = stream.write_all(b"show");
            eprintln!("Eye2020 is already running — bringing window to front.");
            process::exit(0);
        }
        let _ = fs::remove_file(&sock);
    }

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

    if let Ok(mut file) = fs::File::create(&lock_path) {
        let _ = file.write_all(process::id().to_string().as_bytes());
    }
}

#[cfg(unix)]
fn start_ipc_listener(app_handle: tauri::AppHandle) {
    use std::os::unix::net::UnixListener;

    let sock = socket_path();
    let _ = fs::remove_file(&sock);

    let listener = match UnixListener::bind(&sock) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind IPC socket: {}", e);
            return;
        }
    };

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

#[cfg(unix)]
fn cleanup_lock() {
    let _ = fs::remove_file(lock_file_path());
    let _ = fs::remove_file(socket_path());
}

// ── Windows single-instance via lock file only ──

#[cfg(windows)]
fn check_single_instance() {
    let lock_path = lock_file_path();

    if lock_path.exists() {
        // Simple lock file check — if file exists and process is alive, exit
        if let Ok(mut file) = fs::File::open(&lock_path) {
            let mut pid_str = String::new();
            if file.read_to_string(&mut pid_str).is_ok() {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    // Check if process is still running via tasklist
                    let status = process::Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                        .output();
                    if let Ok(output) = status {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if stdout.contains(&pid.to_string()) {
                            eprintln!("Eye2020 is already running (PID {}).", pid);
                            process::exit(0);
                        }
                    }
                }
            }
        }
        let _ = fs::remove_file(&lock_path);
    }

    if let Ok(mut file) = fs::File::create(&lock_path) {
        let _ = file.write_all(process::id().to_string().as_bytes());
    }
}

#[cfg(windows)]
fn start_ipc_listener(_app_handle: tauri::AppHandle) {
    // On Windows, single-instance is handled via lock file only
}

#[cfg(windows)]
fn cleanup_lock() {
    let _ = fs::remove_file(lock_file_path());
}

/// Create a symlink in /usr/local/bin so the app can be launched via `eye2020` from terminal.
/// Uses osascript to prompt for admin password if needed. Only runs once (skips if symlink exists).
fn install_cli_symlink() {
    let symlink_path = std::path::Path::new("/usr/local/bin/eye2020");
    let app_binary = "/Applications/Eye2020.app/Contents/MacOS/Eye2020";

    // Only create if the .app bundle exists
    if !std::path::Path::new(app_binary).exists() {
        return;
    }

    // Already exists and points correctly — nothing to do
    if symlink_path.exists() {
        if let Ok(target) = fs::read_link(symlink_path) {
            if target.to_str() == Some(app_binary) {
                return;
            }
        }
    }

    // Try without sudo first
    #[cfg(unix)]
    {
        if std::os::unix::fs::symlink(app_binary, symlink_path).is_ok() {
            return;
        }

        // Need elevated permissions — use osascript for macOS password prompt
        let script = format!(
            "do shell script \"ln -sf '{}' '{}'\" with administrator privileges",
            app_binary,
            symlink_path.display()
        );
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .output();
    }
}

fn main() {
    check_single_instance();

    let state = app_state::new_shared_state();

    // Load persisted settings from SQLite
    {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(commands::load_persisted_settings(&state));
    }

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
            commands::get_stats,
            commands::set_auto_start,
            commands::toggle_sound,
            commands::quit_app,
            commands::reset_stats,
            commands::play_complete_sound,
            commands::open_url,
            commands::get_theme,
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

            // Create CLI symlink so `eye2020` works from terminal
            install_cli_symlink();

            // Setup tray icon and menu
            ui::tray::setup_tray(&handle, state_clone.clone())?;

            // Setup screen monitoring
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

            // Start IPC listener for single-instance
            start_ipc_listener(handle.clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building Eye2020")
        .run(|app, event| {
            #[allow(unused_variables)]
            match event {
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { has_visible_windows, .. } => {
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
