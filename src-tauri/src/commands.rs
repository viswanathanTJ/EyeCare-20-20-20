use crate::app::app_state::{ReminderMode, SharedAppState};
use crate::app::break_manager;
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
    break_manager::finish_break(&state).await;
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
    Ok(())
}

#[tauri::command]
pub async fn toggle_timer_in_menu(state: State<'_, SharedAppState>) -> Result<bool, String> {
    let mut s = state.lock().await;
    s.show_timer_in_menu = !s.show_timer_in_menu;
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
    Ok(())
}

#[tauri::command]
pub async fn set_break_duration(state: State<'_, SharedAppState>, seconds: u64) -> Result<(), String> {
    if seconds < 5 || seconds > 300 {
        return Err("Break duration must be between 5 and 300 seconds".into());
    }
    let mut s = state.lock().await;
    s.break_duration_secs = seconds;
    Ok(())
}
