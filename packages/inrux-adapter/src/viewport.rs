//! Shared logical viewport state; hosts report dimensions and the coordinator tracks reads.
use inrux_core::reactive::Signal;
use inrux_style::Viewport;
/// A cheap shared handle provided to every component through its Scope.
#[derive(Clone)]
pub struct ViewportState(Signal<Viewport>);
impl Default for ViewportState {
    /// Start without guessing a platform viewport size.
    fn default() -> Self {
        Self(Signal::new(Viewport::default()))
    }
}
impl ViewportState {
    /// Track viewport reads during style or component resolution.
    pub fn get(&self) -> Viewport {
        self.0.get()
    }
    /// Report actual logical dimensions; identical values do not invalidate anything.
    pub fn set(&self, width: f32, height: f32) {
        if width.is_finite() && height.is_finite() && width >= 0.0 && height >= 0.0 {
            self.0.set(Viewport { width, height });
        }
    }
}
