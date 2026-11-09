//! Pure controlled-input reconciliation independent of native focus and painting.
use std::collections::VecDeque;
mod viewport;
pub use viewport::TextViewport;

/// Bounded local edit history shared by backend-specific text controls.
#[derive(Clone)]
pub struct Editor {
    /// Current local editing buffer, reconciled against acknowledged model values.
    pub value: String,
    remote: String,
    applied: u64,
    next: u64,
    pending: VecDeque<(u64, String)>,
}
impl Editor {
    /// Start from the rendered model, resuming after its last acknowledgement.
    pub fn new(value: &str, applied: Option<u64>) -> Self {
        Self {
            value: value.into(),
            remote: value.into(),
            applied: applied.unwrap_or(0),
            next: applied.unwrap_or(0),
            pending: VecDeque::new(),
        }
    }
    /// Accept external replacements even while focused, preserving only newer queued edits.
    pub fn reconcile(&mut self, value: &str, applied: Option<u64>) {
        if let Some(applied) = applied {
            if applied < self.applied {
                return;
            }
            self.applied = applied;
            self.next = self.next.max(applied);
            self.pending.retain(|(revision, _)| *revision > applied);
        } else if self.remote != value {
            // Plain callback nodes cannot acknowledge normalized edits; match exact values only.
            if let Some(index) = self
                .pending
                .iter()
                .position(|(_, pending)| pending == value)
            {
                self.pending.drain(..=index);
            } else {
                // A new value absent from local history is an external replacement.
                self.pending.clear();
            }
        }
        self.remote = value.into();
        self.value = self
            .pending
            .back()
            .map_or_else(|| value.to_owned(), |(_, value)| value.clone());
    }
    /// Assign an editing revision before attempting the nonblocking dispatch.
    pub fn edited(&mut self) -> u64 {
        self.next = self.next.wrapping_add(1);
        // Legacy plain events cannot identify every coalesced acknowledgement. Bound
        // their retained history to the native admission window, keeping the newest edit.
        if self.pending.len() >= 65 {
            self.pending.pop_front();
        }
        self.pending.push_back((self.next, self.value.clone()));
        self.next
    }
    /// A full/stopped command queue must not display a rejected model change.
    pub fn reject(&mut self, previous: String) {
        self.pending.pop_back();
        self.value = previous;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// An old acknowledgement must preserve the more recent local edit across blur/focus changes.
    #[test]
    fn old_acknowledgement_preserves_newer_pending_edit() {
        let mut editor = Editor::new("a", Some(0));
        editor.value = "ab".into();
        editor.edited();
        editor.value = "abc".into();
        editor.edited();
        editor.reconcile("ab", Some(1));
        assert_eq!(editor.value, "abc");
        editor.reconcile("ABC", Some(2));
        assert_eq!(editor.value, "ABC");
    }
    /// Focus does not mask a programmatic reset after all earlier edits have been acknowledged.
    #[test]
    fn external_reset_and_rejected_edit_restore_the_controlled_model() {
        let mut editor = Editor::new("original", Some(0));
        editor.reconcile("reset", Some(0));
        assert_eq!(editor.value, "reset");
        editor.value = "rejected".into();
        editor.edited();
        editor.reject("reset".into());
        editor.reconcile("reset", Some(0));
        assert_eq!(editor.value, "reset");
    }
}
