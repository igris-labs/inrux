//! Bounded background work, separate from reactive updates and rendering.
mod cancellation;
mod executor;
pub use cancellation::{Cancellation, TaskHandle};
pub(crate) use executor::Executor;
pub use executor::{TaskError, TaskQueueError};
