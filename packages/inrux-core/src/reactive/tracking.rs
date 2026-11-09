//! Nested read tracking and subscription ownership.
use super::ReactiveError;
use std::{cell::RefCell, sync::Arc};

/// Source identity and revision, read atomically with the source value.
#[derive(Clone)]
pub(crate) struct Observed {
    pub(crate) source: Arc<dyn Dependency>,
    pub(crate) version: u64,
}
/// A versioned dependency; computed dependencies can refresh lazily.
pub(crate) trait Dependency: Send + Sync {
    /// Globally unique process-local identity.
    fn id(&self) -> u64;
    /// Current revision, evaluating derived state when needed.
    fn revision(&self) -> Result<u64, ReactiveError>;
    /// Subscribe to underlying source changes using a weak notification handle.
    fn subscribe(&self, callback: &Arc<dyn Fn() + Send + Sync>);
}
thread_local! { static READS: RefCell<Vec<Vec<Observed>>> = const { RefCell::new(Vec::new()) }; }

/// Keep a notification alive for exactly as long as its owning component/view.
pub struct Subscription {
    _callback: Arc<dyn Fn() + Send + Sync>,
}
/// The sources read during one tracked evaluation.
#[derive(Clone, Default)]
pub struct Dependencies {
    pub(crate) reads: Vec<Observed>,
}
impl Dependencies {
    /// Check revisions without rerunning the caller's view or pure computation.
    pub fn is_stale(&self) -> Result<bool, ReactiveError> {
        for observed in &self.reads {
            if observed.source.revision()? != observed.version {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Notify while this subscription is alive; old subscriptions disappear without global leaks.
    pub fn subscribe(&self, callback: impl Fn() + Send + Sync + 'static) -> Subscription {
        let callback: Arc<dyn Fn() + Send + Sync> = Arc::new(callback);
        for observed in &self.reads {
            observed.source.subscribe(&callback);
        }
        // Close the read/subscribe race: a write between tracking and registration
        // must schedule another view, even when no later writes arrive.
        if self.is_stale().unwrap_or(true) {
            callback();
        }
        Subscription {
            _callback: callback,
        }
    }
}
/// Restore the parent tracker even when user computation unwinds.
struct ReadGuard;
impl Drop for ReadGuard {
    /// Drop only the nested scope left by this evaluation.
    fn drop(&mut self) {
        READS.with(|reads| {
            reads.borrow_mut().pop();
        });
    }
}
/// Evaluate a function once while recording its reactive reads, including conditional reads.
pub fn track<T>(evaluate: impl FnOnce() -> T) -> (T, Dependencies) {
    READS.with(|reads| reads.borrow_mut().push(vec![]));
    let guard = ReadGuard;
    let value = evaluate();
    let reads = READS.with(|reads| reads.borrow().last().cloned().unwrap_or_default());
    drop(guard);
    (value, Dependencies { reads })
}
/// Add the revision observed by a value read to the current nested tracking scope.
pub(crate) fn record(source: Arc<dyn Dependency>, version: u64) {
    READS.with(|reads| {
        if let Some(active) = reads.borrow_mut().last_mut()
            && !active.iter().any(|entry| entry.source.id() == source.id())
        {
            active.push(Observed { source, version });
        }
    });
}
