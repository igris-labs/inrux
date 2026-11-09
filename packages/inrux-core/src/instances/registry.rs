//! Retained state storage; removal invalidates every outstanding handle before slot reuse.
use super::InstanceHandle;
use std::{
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
static NEXT_REGISTRY: AtomicU64 = AtomicU64::new(1);
struct Slot<T> {
    generation: u64,
    value: Option<T>,
    cancelled: Arc<AtomicBool>,
}
struct Storage<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
}
impl<T> Drop for Storage<T> {
    /// Registry teardown cancels owned resources even if a handle outlives the root.
    fn drop(&mut self) {
        for slot in &self.slots {
            slot.cancelled.store(true, Ordering::Release);
        }
    }
}
/// Retained component/composable state; clones share the same registry rather than duplicating instances.
///
/// Callbacks run under a short storage lock. Do not reenter this registry or block inside them.
/// Schedule long work with Context::spawn_owned_task using owned inputs instead.
pub struct Instances<T> {
    id: u64,
    storage: Arc<Mutex<Storage<T>>>,
}
impl<T> Clone for Instances<T> {
    /// Preserve a single owner across immutable root snapshots.
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            storage: self.storage.clone(),
        }
    }
}
impl<T> Default for Instances<T> {
    /// Create an empty registry with process-unique identity.
    fn default() -> Self {
        Self {
            id: NEXT_REGISTRY.fetch_add(1, Ordering::Relaxed),
            storage: Arc::new(Mutex::new(Storage {
                slots: vec![],
                free: vec![],
            })),
        }
    }
}
impl<T> Instances<T> {
    /// Mount owned state exactly once and return its typed identity.
    pub fn mount(&self, value: T) -> InstanceHandle<T> {
        let mut storage = self
            .storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = if let Some(index) = storage.free.pop() {
            let slot = &mut storage.slots[index];
            slot.value = Some(value);
            slot.cancelled = Arc::new(AtomicBool::new(false));
            index
        } else {
            let index = storage.slots.len();
            storage.slots.push(Slot {
                generation: 0,
                value: Some(value),
                cancelled: Arc::new(AtomicBool::new(false)),
            });
            index
        };
        let slot = &storage.slots[index];
        InstanceHandle {
            registry: self.id,
            slot: index,
            generation: slot.generation,
            cancelled: slot.cancelled.clone(),
            marker: PhantomData,
        }
    }
    /// Access retained state only if the registry and mount generation still match.
    pub fn with<R>(&self, handle: &InstanceHandle<T>, read: impl FnOnce(&T) -> R) -> Option<R> {
        if handle.registry != self.id || !handle.is_mounted() {
            return None;
        }
        let storage = self
            .storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slot = storage.slots.get(handle.slot)?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.value.as_ref().map(read)
    }
    /// Mutate retained state without leaking a mutable borrow across task boundaries.
    pub fn update<R>(
        &self,
        handle: &InstanceHandle<T>,
        update: impl FnOnce(&mut T) -> R,
    ) -> Option<R> {
        if handle.registry != self.id || !handle.is_mounted() {
            return None;
        }
        let mut storage = self
            .storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slot = storage.slots.get_mut(handle.slot)?;
        if slot.generation != handle.generation {
            return None;
        }
        slot.value.as_mut().map(update)
    }
    /// Unmount before reusing a slot; stale messages and owned task results become harmless.
    pub fn unmount(&self, handle: &InstanceHandle<T>) -> Option<T> {
        if handle.registry != self.id {
            return None;
        }
        let mut storage = self
            .storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let slot = storage.slots.get_mut(handle.slot)?;
        if slot.generation != handle.generation {
            return None;
        }
        let value = slot.value.take()?;
        slot.cancelled.store(true, Ordering::Release);
        if let Some(generation) = slot.generation.checked_add(1) {
            slot.generation = generation;
            storage.free.push(handle.slot);
        }
        Some(value)
    }
}
