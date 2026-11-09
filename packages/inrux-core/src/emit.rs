//! Typed component event delivery.
use crate::DispatchError;
use std::sync::Arc;
/// A typed child-to-parent event. Callbacks enqueue work; they must not do blocking work.
pub struct Emit<T>(Arc<dyn Fn(T) -> Result<(), DispatchError> + Send + Sync>);
impl<T> Clone for Emit<T> {
    /// Share the immutable typed callback without requiring the payload to be Clone.
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> Emit<T> {
    /// Construct an event from a nonblocking callback.
    pub fn new(f: impl Fn(T) -> Result<(), DispatchError> + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }
    /// Deliver an event, reporting backpressure or a stopped worker.
    pub fn emit(&self, value: T) -> Result<(), DispatchError> {
        (self.0)(value)
    }
}
