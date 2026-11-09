//! Explicit task cancellation combined with application shutdown.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
/// Cancellation view passed to a running task. Blocking work must cooperate.
#[derive(Clone)]
pub struct Cancellation {
    pub(crate) application: Arc<AtomicBool>,
    pub(crate) local: Arc<AtomicBool>,
    pub(crate) parent: Option<Arc<AtomicBool>>,
    pub(crate) owner: Option<Arc<AtomicBool>>,
}
impl Cancellation {
    /// Check local cancellation or application teardown without touching component state.
    pub fn is_cancelled(&self) -> bool {
        self.local.load(Ordering::Acquire)
            || self.application.load(Ordering::Acquire)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.load(Ordering::Acquire))
            || self
                .owner
                .as_ref()
                .is_some_and(|owner| owner.load(Ordering::Acquire))
    }
}
/// An owned handle used to explicitly cancel a task.
#[derive(Clone)]
pub struct TaskHandle {
    pub(crate) token: Cancellation,
}
impl TaskHandle {
    /// Request cooperative cancellation. Cancelled results are never applied to state.
    pub fn cancel(&self) {
        self.token.local.store(true, Ordering::Release);
    }
    /// Inspect whether the task's owner has cancelled it.
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
}
