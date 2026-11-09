//! Thread-safe values whose reads are tracked and whose writes invalidate subscribers.
use super::{Dependency, ReactiveError, record};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicU64, Ordering},
};
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
/// Allocate identity independently of widget positions and state values.
pub(crate) fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
type Callback = dyn Fn() + Send + Sync;
struct Inner<T> {
    id: u64,
    data: Mutex<(T, u64)>,
    listeners: Mutex<Vec<Weak<Callback>>>,
}
/// An explicitly reactive, owned value; cloning a handle deliberately shares that value.
pub struct Signal<T> {
    inner: Arc<Inner<T>>,
}
impl<T> Clone for Signal<T> {
    /// Clone the shared handle, not a second independent state instance.
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Create independent state. Call in component setup, not on every render.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Inner {
                id: next_id(),
                data: Mutex::new((value, 0)),
                listeners: Mutex::new(vec![]),
            }),
        }
    }
    /// Read a value and track the exact revision associated with that read.
    pub fn get(&self) -> T {
        let (value, version) = self
            .inner
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        record(self.inner.clone(), version);
        value
    }
    /// Mutate in one transaction, waking subscribers after the value lock is released.
    pub fn update(&self, update: impl FnOnce(&mut T))
    where
        T: PartialEq,
    {
        let changed = {
            let mut data = self
                .inner
                .data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let before = data.0.clone();
            update(&mut data.0);
            if before != data.0 {
                data.1 = data.1.wrapping_add(1);
                true
            } else {
                false
            }
        };
        if changed {
            let callbacks = {
                let mut listeners = self
                    .inner
                    .listeners
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let callbacks = listeners
                    .iter()
                    .filter_map(Weak::upgrade)
                    .collect::<Vec<_>>();
                listeners.retain(|entry| entry.strong_count() > 0);
                callbacks
            };
            for callback in callbacks {
                callback();
            }
        }
    }
    /// Replace state, avoiding invalidation when the value is unchanged.
    pub fn set(&self, value: T)
    where
        T: PartialEq,
    {
        self.update(|current| *current = value);
    }
}
impl<T: Clone + Send + Sync + 'static> Dependency for Inner<T> {
    /// Stable source identity for deduplicating reads.
    fn id(&self) -> u64 {
        self.id
    }
    /// Source revisions do not require evaluating application code.
    fn revision(&self) -> Result<u64, ReactiveError> {
        Ok(self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .1)
    }
    /// Keep weak subscriptions so unmounted views are not retained by shared state.
    fn subscribe(&self, callback: &Arc<Callback>) {
        let mut listeners = self
            .listeners
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Dormant sources may never write again; prune during registration too. A
        // direct read plus a computed read can subscribe the same callback twice.
        listeners.retain(|listener| listener.strong_count() > 0);
        let listener = Arc::downgrade(callback);
        if !listeners.iter().any(|existing| existing.ptr_eq(&listener)) {
            listeners.push(listener);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Repeated renders must not accumulate expired listeners on an unchanged shared source.
    #[test]
    fn unchanged_sources_prune_disposed_view_subscriptions() {
        let signal = Signal::new(0);
        for _ in 0..100 {
            let (_, dependencies) = crate::reactive::track(|| signal.get());
            let _subscription = dependencies.subscribe(|| {});
        }
        assert_eq!(signal.inner.listeners.lock().unwrap().len(), 1);
    }
}
