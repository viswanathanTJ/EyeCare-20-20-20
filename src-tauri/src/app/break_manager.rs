use super::app_state::{MonitoringStatus, SharedAppState};
use crate::data::stats_store::StatsStore;
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

fn refresh_stats(s: &mut super::app_state::AppState) {
    if let Ok(store) = StatsStore::open() {
        let (completed, skipped) = store.get_today_stats();
        let (current, longest) = store.get_streak();
        s.today_completed = completed;
        s.today_skipped = skipped;
        s.current_streak = current;
        s.longest_streak = longest;
        s.total_breaks = store.get_total_breaks();
    }
}

/// Called when the user completes a break.
pub async fn finish_break(state: &SharedAppState) {
    if let Ok(store) = StatsStore::open() {
        let _ = store.record_break_completed();
    }
    let mut s = state.lock().await;
    s.seconds_remaining = s.break_interval_secs;
    s.status = MonitoringStatus::Active;
    s.snooze_count = 0;
    s.snooze_remaining = 0;
    refresh_stats(&mut s);
}

/// Called when the user skips a break.
pub async fn skip_break(state: &SharedAppState) {
    if let Ok(store) = StatsStore::open() {
        let _ = store.record_break_skipped();
    }
    let mut s = state.lock().await;
    s.seconds_remaining = s.break_interval_secs;
    s.status = MonitoringStatus::Active;
    s.snooze_count = 0;
    s.snooze_remaining = 0;
    refresh_stats(&mut s);
}

/// Called when the user (or auto-snooze) snoozes a break.
/// Returns Err if max snoozes reached.
pub async fn snooze_break(state: &SharedAppState) -> Result<(), String> {
    let mut s = state.lock().await;
    if s.snooze_count >= s.max_snoozes {
        return Err("Maximum snoozes reached".into());
    }
    s.snooze_count += 1;
    s.snooze_remaining = s.snooze_duration_secs;
    s.status = MonitoringStatus::Snoozed;
    Ok(())
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
    s.snooze_count = 0;
    s.snooze_remaining = 0;
}
