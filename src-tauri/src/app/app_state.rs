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
    /// Today's completed breaks
    pub today_completed: i64,
    /// Today's skipped breaks
    pub today_skipped: i64,
    /// Current streak (consecutive days with breaks)
    pub current_streak: i64,
    /// Longest streak ever
    pub longest_streak: i64,
    /// Total breaks all-time
    pub total_breaks: i64,
    /// Whether to play sound on break reminder
    pub sound_enabled: bool,
    /// Whether auto-start on login is enabled
    pub auto_start: bool,
    /// Current system appearance: "dark" or "light"
    pub theme: String,
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
            today_completed: 0,
            today_skipped: 0,
            current_streak: 0,
            longest_streak: 0,
            total_breaks: 0,
            sound_enabled: true,
            auto_start: false,
            theme: "dark".to_string(),
        }
    }
}

pub type SharedAppState = Arc<Mutex<AppState>>;

pub fn new_shared_state() -> SharedAppState {
    Arc::new(Mutex::new(AppState::default()))
}
