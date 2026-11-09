//! Reactive state, typed events, scoped dependencies, and application scheduling.
#![deny(missing_docs)]
mod emit;
mod error;
pub mod reactive;
mod scope;
pub mod tasks;
mod worker;
pub use emit::Emit;
pub use error::DispatchError;
pub use scope::Scope;
pub use worker::{Context, Sender, Worker};
#[cfg(test)]
mod tests;

pub mod instances;
pub use instances::{InstanceHandle, Instances};

mod instance_dispatch;
