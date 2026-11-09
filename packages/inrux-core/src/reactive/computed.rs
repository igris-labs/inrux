//! Lazy cached computation with nested dependency tracking and cycle reporting.
use super::signal::next_id;
use super::{Dependencies, Dependency, record, track};
use std::{
    cell::RefCell,
    collections::HashSet,
    fmt,
    sync::{Arc, Mutex, TryLockError},
};
thread_local! { static ACTIVE:RefCell<HashSet<u64>>=RefCell::new(HashSet::new()); }
/// A computed read could not safely complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactiveError {
    /// A getter recursively reads its own dependency graph.
    Cycle,
    /// Another thread is evaluating this value; retry on the coordinator.
    Busy,
}
impl fmt::Display for ReactiveError {
    /// Explain cycles without exposing an unchecked recursive evaluation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cycle => "Computed dependency cycle detected",
            Self::Busy => "Computed value is being evaluated on another thread",
        })
    }
}
impl std::error::Error for ReactiveError {}
struct Cache<T> {
    value: T,
    dependencies: Dependencies,
    revision: u64,
}
struct Inner<T> {
    id: u64,
    evaluate: Box<dyn Fn() -> T + Send + Sync>,
    cache: Mutex<Option<Cache<T>>>,
}
/// An owned, pure derived value; dependencies are discovered automatically on reads.
pub struct Computed<T> {
    inner: Arc<Inner<T>>,
}
impl<T> Clone for Computed<T> {
    /// Share a computed cache across readers of the same component value.
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
/// Remove the recursion marker even if user code panics.
struct ActiveGuard(u64);
impl Drop for ActiveGuard {
    /// Restore the caller's evaluation stack.
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.borrow_mut().remove(&self.0);
        });
    }
}
impl<T: Clone + PartialEq + Send + Sync + 'static> Computed<T> {
    /// Define a getter without executing it until its value is requested.
    pub fn new(evaluate: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self {
            inner: Arc::new(Inner {
                id: next_id(),
                evaluate: Box::new(evaluate),
                cache: Mutex::new(None),
            }),
        }
    }
    /// Read lazily, returning an explicit error if evaluation forms a dependency cycle.
    pub fn get(&self) -> Result<T, ReactiveError> {
        match self.inner.read() {
            Ok((value, revision)) => {
                record(self.inner.clone(), revision);
                Ok(value)
            }
            Err(ReactiveError::Busy) => {
                // A temporary fallback must not lose the dependency and remain frozen
                // after another reader finishes the getter. Computed revisions start at one.
                record(self.inner.clone(), 0);
                Err(ReactiveError::Busy)
            }
            Err(error) => Err(error),
        }
    }
}
impl<T: Clone + PartialEq + Send + Sync + 'static> Inner<T> {
    /// Validate cached dependency revisions before evaluating the getter again.
    fn read(&self) -> Result<(T, u64), ReactiveError> {
        let entered = ACTIVE.with(|active| active.borrow_mut().insert(self.id));
        if !entered {
            return Err(ReactiveError::Cycle);
        }
        let _guard = ActiveGuard(self.id);
        // Never wait while traversing a dependency graph: two cross-thread getters
        // may otherwise each hold the cache needed by the other.
        let mut cache = match self.cache.try_lock() {
            Ok(cache) => cache,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(ReactiveError::Busy),
        };
        if let Some(cached) = cache.as_ref()
            && !cached.dependencies.is_stale()?
        {
            return Ok((cached.value.clone(), cached.revision));
        }
        let (value, dependencies) = track(|| (self.evaluate)());
        let revision = cache.as_ref().map_or(1, |cached| {
            cached
                .revision
                .wrapping_add(u64::from(cached.value != value))
        });
        *cache = Some(Cache {
            value: value.clone(),
            dependencies,
            revision,
        });
        Ok((value, revision))
    }
}
impl<T: Clone + PartialEq + Send + Sync + 'static> Dependency for Inner<T> {
    /// Unique identity for nested cached reads.
    fn id(&self) -> u64 {
        self.id
    }
    /// Refresh a derived dependency before comparing its semantic revision.
    fn revision(&self) -> Result<u64, ReactiveError> {
        self.read().map(|(_, revision)| revision)
    }
    /// Forward changes from the current dependency set; view reevaluation replaces branch subscriptions.
    fn subscribe(&self, callback: &Arc<dyn Fn() + Send + Sync>) {
        // A branch may change between the caller's read and subscription, even
        // when the computed result stays equal. Refresh before selecting sources.
        let _ = self.read();
        let dependencies = match self.cache.try_lock() {
            Ok(cache) => cache.as_ref().map(|cache| cache.dependencies.clone()),
            Err(TryLockError::Poisoned(error)) => error
                .into_inner()
                .as_ref()
                .map(|cache| cache.dependencies.clone()),
            Err(TryLockError::WouldBlock) => None,
        };
        if let Some(dependencies) = dependencies {
            for observed in dependencies.reads {
                observed.source.subscribe(callback);
            }
        }
    }
}
