//! UI-thread caret scheduling; no permanent frame loop or ownership cycle.
use super::*;
use std::time::Duration;
impl TextInput {
    /// Editing immediately restores the caret and restarts its bounded native timer.
    pub(super) fn restart_caret(&mut self) {
        self.caret_task = None;
        self.caret_visible = true;
    }
    /// Only an enabled, focused editor owns the native input focus affordance.
    pub(crate) fn has_focus(&self, window: &Window) -> bool {
        !self.disabled && self.focus_handle.is_focused(window)
    }
    /// Match a one-second blink cycle, then stay solid after ten seconds without input.
    pub(super) fn sync_caret(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.has_focus(window) || !window.is_window_active() {
            self.restart_caret();
            return;
        }
        if self.caret_task.is_some() {
            return;
        }
        // Retain the completed task until editing/focus changes so unrelated renders cannot restart blinking.
        self.caret_task = Some(cx.spawn_in(window, async move |input, cx| {
            for tick in 1..=20 {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                let keep_blinking = input
                    .update_in(cx, |input, window, cx| {
                        if !input.has_focus(window) || !window.is_window_active() {
                            return false;
                        }
                        input.caret_visible = tick == 20 || tick % 2 == 0;
                        cx.notify();
                        true
                    })
                    .unwrap_or(false);
                if !keep_blinking {
                    break;
                }
            }
        }));
    }
}
