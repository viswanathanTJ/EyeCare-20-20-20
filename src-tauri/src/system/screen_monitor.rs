use crate::app::app_state::SharedAppState;
use tauri::AppHandle;

/// Sets up macOS workspace notifications to detect screen sleep/wake.
/// Uses CFRunLoop + NSWorkspace distributed notification center.
///
/// For Phase 1, this is a placeholder — the timer runs unconditionally.
/// Phase 6 will add real IOKit / NSWorkspace screen event detection.
pub fn setup_screen_monitoring(_app: &AppHandle, _state: SharedAppState) {
    // TODO (Phase 6): Register for these NSWorkspace notifications:
    // - NSWorkspaceScreensDidSleepNotification
    // - NSWorkspaceScreensDidWakeNotification
    // - NSWorkspaceSessionDidBecomeActiveNotification
    // - NSWorkspaceSessionDidResignActiveNotification
    //
    // On sleep/lock → pause timer
    // On wake/unlock → resume timer
}
