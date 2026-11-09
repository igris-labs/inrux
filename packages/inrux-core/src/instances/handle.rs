//! Typed instance identity survives reordering and becomes invalid on unmount.
use std::{
    fmt,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
/// A typed reference to one mount generation; it does not keep component state alive.
pub struct InstanceHandle<T> {
    pub(crate) registry: u64,
    pub(crate) slot: usize,
    pub(crate) generation: u64,
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) marker: PhantomData<fn() -> T>,
}
impl<T> Clone for InstanceHandle<T> {
    /// Share identity without requiring component state to implement Clone.
    fn clone(&self) -> Self {
        Self {
            registry: self.registry,
            slot: self.slot,
            generation: self.generation,
            cancelled: self.cancelled.clone(),
            marker: PhantomData,
        }
    }
}
impl<T> InstanceHandle<T> {
    /// Whether unmount or registry disposal has invalidated this generation.
    pub fn is_mounted(&self) -> bool {
        !self.cancelled.load(Ordering::Acquire)
    }
}
impl<T> fmt::Display for InstanceHandle<T> {
    /// Include the generation in stable widget keys so remounts cannot reuse stale editor state.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.registry, self.slot, self.generation)
    }
}
