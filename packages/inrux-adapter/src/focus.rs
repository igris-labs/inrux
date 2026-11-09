//! Focus paint policy shared by native hosts; logical focus is retained for all input sources.
/// Last interaction modality, used for focus-visible decoration independently of focus ownership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusVisibility(bool);
impl Default for FocusVisibility {
    /// Programmatic/accessibility focus starts visible until an actual pointer interaction occurs.
    fn default() -> Self {
        Self(true)
    }
}
impl FocusVisibility {
    /// Pointer activation keeps focus and keyboard activation working, without a keyboard outline.
    pub fn pointer(&mut self) -> bool {
        let changed = self.0;
        self.0 = false;
        changed
    }
    /// Keyboard interaction restores the focus-visible outline, including on an already focused control.
    pub fn keyboard(&mut self) -> bool {
        let changed = !self.0;
        self.0 = true;
        changed
    }
    /// Whether a focused non-editor control should draw keyboard focus feedback.
    pub fn visible(self) -> bool {
        self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    /// Pointer selection and keyboard focus are independent from logical focused-widget identity.
    #[test]
    fn pointer_hides_outline_and_keyboard_restores_it_without_losing_focus() {
        let mut visibility = FocusVisibility::default();
        assert!(visibility.visible());
        assert!(visibility.pointer());
        assert!(!visibility.visible());
        assert!(!visibility.pointer());
        assert!(visibility.keyboard());
        assert!(visibility.visible());
    }
}
