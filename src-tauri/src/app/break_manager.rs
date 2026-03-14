use super::app_state::{MonitoringStatus, SharedAppState};
use rand::seq::SliceRandom;

const TIPS: &[&str] = &[
    "Look at something 20 feet away.",
    "Relax your eye muscles.",
    "Blink slowly for a few seconds.",
    "Blink often to keep eyes hydrated.",
    "Look far away to relax eye muscles.",
    "Short breaks protect long-term vision.",
    "Close your eyes and take a deep breath.",
    "Roll your eyes gently in circles.",
    "Focus on a distant object for 20 seconds.",
    "Cup your palms over your eyes to rest them.",
];

pub fn random_tip() -> &'static str {
    let mut rng = rand::thread_rng();
    TIPS.choose(&mut rng).unwrap_or(&TIPS[0])
}

/// Called when the user completes or skips a break.
/// Resets the timer and returns to Active monitoring.
pub async fn finish_break(state: &SharedAppState) {
    let mut s = state.lock().await;
    s.seconds_remaining = s.break_interval_secs;
    s.status = MonitoringStatus::Active;
}

/// Called when the user wants to pause monitoring.
pub async fn pause_monitoring(state: &SharedAppState) {
    let mut s = state.lock().await;
    s.status = MonitoringStatus::Paused;
}

/// Called when the user wants to resume monitoring.
pub async fn resume_monitoring(state: &SharedAppState) {
    let mut s = state.lock().await;
    s.status = MonitoringStatus::Active;
}

/// Force trigger a break immediately.
pub async fn take_break_now(state: &SharedAppState) {
    let mut s = state.lock().await;
    s.seconds_remaining = 0;
    s.status = MonitoringStatus::OnBreak;
}
