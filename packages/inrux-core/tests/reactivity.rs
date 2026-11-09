//! Observable-state semantics independent of native widgets.
use inrux_core::reactive::{Computed, Signal, track};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Unrelated writes and repeated frames must not reevaluate a pure getter.
#[test]
fn computed_tracks_dependencies_and_caches_until_they_change() {
    let value = Signal::new(2);
    let unrelated = Signal::new(0);
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let source = value.clone();
    let doubled = Computed::new(move || {
        counter.fetch_add(1, Ordering::SeqCst);
        source.get() * 2
    });
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(doubled.get(), Ok(4));
    assert_eq!(doubled.get(), Ok(4));
    unrelated.set(10);
    assert_eq!(doubled.get(), Ok(4));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    value.set(3);
    assert_eq!(doubled.get(), Ok(6));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

/// Conditional reads replace old dependencies instead of growing a permanent subscription list.
#[test]
fn computed_changes_dependencies_when_a_branch_changes() {
    let choose = Signal::new(true);
    let a = Signal::new(1);
    let b = Signal::new(2);
    let (c, x, y) = (choose.clone(), a.clone(), b.clone());
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let value = Computed::new(move || {
        count.fetch_add(1, Ordering::SeqCst);
        if c.get() { x.get() } else { y.get() }
    });
    assert_eq!(value.get(), Ok(1));
    b.set(4);
    assert_eq!(value.get(), Ok(1));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    choose.set(false);
    assert_eq!(value.get(), Ok(4));
    a.set(9);
    assert_eq!(value.get(), Ok(4));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

/// Disposing a view releases its wake callback even if the source store outlives it.
#[test]
fn subscriptions_end_when_the_owner_is_dropped() {
    let signal = Signal::new(1);
    let (_, dependencies) = track(|| signal.get());
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let subscription = dependencies.subscribe(move || {
        count.fetch_add(1, Ordering::SeqCst);
    });
    signal.set(2);
    assert!(dependencies.is_stale().unwrap());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(subscription);
    signal.set(3);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// Equal derived results do not invalidate a parent computed getter.
#[test]
fn nested_computed_values_compare_semantic_revisions() {
    let source = Signal::new(2);
    let input = source.clone();
    let even = Computed::new(move || input.get() % 2 == 0);
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let derived = Computed::new(move || {
        count.fetch_add(1, Ordering::SeqCst);
        even.get().unwrap_or(false)
    });
    assert_eq!(derived.get(), Ok(true));
    source.set(4);
    assert_eq!(derived.get(), Ok(true));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// Writes during the gap between rendering and subscribing cannot leave a stale view forever.
#[test]
fn subscription_detects_a_write_that_happened_after_tracking() {
    let source = Signal::new(0);
    let (_, dependencies) = inrux_core::reactive::track(|| source.get());
    source.set(1);
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let listener = calls.clone();
    let _subscription = dependencies.subscribe(move || {
        listener.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}

/// Cross-thread getter contention returns an error instead of deadlocking two dependency graphs.
#[test]
fn concurrent_computed_read_does_not_wait_on_an_in_progress_getter() {
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let release_rx = std::sync::Mutex::new(release_rx);
    let computed = Computed::new(move || {
        started_tx.send(()).unwrap();
        release_rx.lock().unwrap().recv().unwrap();
        1
    });
    let other = computed.clone();
    let reader = std::thread::spawn(move || other.get());
    started_rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .unwrap();
    let (result, dependencies) = inrux_core::reactive::track(|| computed.get());
    assert_eq!(result, Err(inrux_core::reactive::ReactiveError::Busy));
    release_tx.send(()).unwrap();
    assert_eq!(reader.join().unwrap(), Ok(1));
    assert!(dependencies.is_stale().unwrap());
}

/// A source reached both directly and through computed dependencies wakes a view only once.
#[test]
fn overlapping_dependency_paths_do_not_duplicate_notifications() {
    let source = Signal::new(1);
    let derived = source.clone();
    let computed = Computed::new(move || derived.get() * 2);
    let (_, dependencies) = inrux_core::reactive::track(|| (source.get(), computed.get().unwrap()));
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let callback = count.clone();
    let _subscription = dependencies.subscribe(move || {
        callback.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    source.set(2);
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

/// Equal computed results must still retarget subscriptions when a branch changes before registration.
#[test]
fn registration_tracks_new_branch_even_when_computed_value_is_equal() {
    let choose_first = Signal::new(true);
    let first = Signal::new(1);
    let second = Signal::new(1);
    let choose = choose_first.clone();
    let left = first.clone();
    let right = second.clone();
    let computed = Computed::new(move || {
        if choose.get() {
            left.get()
        } else {
            right.get()
        }
    });
    let (_, dependencies) = inrux_core::reactive::track(|| computed.get().unwrap());
    choose_first.set(false);
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let callback = count.clone();
    let _subscription = dependencies.subscribe(move || {
        callback.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    second.set(2);
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

/// Recursive getters receive an explicit cycle error rather than waiting on their own cache lock.
#[test]
fn recursive_computed_read_reports_cycle_to_the_getter() {
    use inrux_core::reactive::ReactiveError;
    let slot = std::sync::Arc::new(std::sync::Mutex::new(None::<Computed<u32>>));
    let weak = std::sync::Arc::downgrade(&slot);
    let observed = std::sync::Arc::new(std::sync::Mutex::new(None));
    let error = observed.clone();
    let computed = Computed::new(move || {
        let owner = weak.upgrade().unwrap();
        let getter = owner.lock().unwrap().as_ref().unwrap().clone();
        *error.lock().unwrap() = getter.get().err();
        0
    });
    *slot.lock().unwrap() = Some(computed.clone());
    assert_eq!(computed.get(), Ok(0));
    assert_eq!(*observed.lock().unwrap(), Some(ReactiveError::Cycle));
}
