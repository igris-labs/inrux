//! Independent counter state shared by multiple reusable component instances.
use inrux::{Computed, Signal};

/// Reactive values owned by one counter in the root composition.
#[derive(Clone)]
pub struct Counter {
    /// Writable source value.
    pub count: Signal<u32>,
    /// Cached derived display value.
    pub doubled: Computed<u32>,
}
/// Create independent state; sharing requires explicitly cloning the returned handles.
pub fn use_counter() -> Counter {
    let count = Signal::new(0);
    let source = count.clone();
    Counter {
        count,
        doubled: Computed::new(move || source.get() * 2),
    }
}
