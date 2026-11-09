//! Explicit retained component state with generation-checked handles and owned cancellation.
mod handle;
mod registry;
pub use handle::InstanceHandle;
pub use registry::Instances;
