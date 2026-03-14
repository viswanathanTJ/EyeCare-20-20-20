use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitoringStatus {
    Active,
    Paused,
    OnBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReminderMode {
    /// Show a break window + notification
    WindowAndNotification,
    /// Notification only (no window)
    NotificationOnly,
    /// Break window only (no notification)
    WindowOnly,
    /// Silent (just reset timer, no UI)
    Silent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub status: MonitoringStatus,
    /// Seconds remaining until next break
    pub seconds_remaining: u64,
    /// Total break interval in seconds (default: 1200 = 20 minutes)
    pub break_interval_secs: u64,
    /// Break duration in seconds (default: 20)
    pub break_duration_secs: u64,
    /// How to remind the user
    pub reminder_mode: ReminderMode,
    /// Whether to show countdown in tray menu title
    pub show_timer_in_menu: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            status: MonitoringStatus::Active,
            seconds_remaining: 1200,
            break_interval_secs: 1200,
            break_duration_secs: 20,
            reminder_mode: ReminderMode::WindowAndNotification,
            show_timer_in_menu: true,
        }
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedAppState {
    Arc::new(Mutex::new(AppState::default()))
}
