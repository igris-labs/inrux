//! Runtime isolation and queue regressions.
use super::*;
use std::{sync::mpsc, thread, time::Duration};

#[test]
fn slow_action_does_not_block_dispatch_or_snapshot_reads() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let worker = Worker::spawn(0_u32, 1, || {}).unwrap();
    let ui_thread = thread::current().id();
    worker
        .sender()
        .dispatch(move |state, _| {
            started_tx.send(thread::current().id()).unwrap();
            release_rx.recv().unwrap();
            *state = 1;
        })
        .unwrap();
    assert_ne!(
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        ui_thread
    );
    assert_eq!(**worker.take_snapshot().as_ref().unwrap(), 0);
    worker
        .sender()
        .dispatch(move |state, ctx| {
            *state += 1;
            ctx.publish(state);
            done_tx.send(()).unwrap();
        })
        .unwrap();
    assert_eq!(
        worker.sender().dispatch(|_, _| {}),
        Err(DispatchError::Full)
    );
    release_tx.send(()).unwrap();
    done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    // The worker may immediately republish after the action returns; reads deliberately use try_lock.
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    let snapshot = loop {
        if let Some(snapshot) = worker.take_snapshot() {
            break snapshot;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "snapshot was never available"
        );
        thread::yield_now();
    };
    assert_eq!(*snapshot, 2);
}
#[test]
fn injection_is_typed_and_nearest_provider_wins() {
    let root = Scope::default().provide(10_u32);
    let child = root.provide(20_u32).provide("service");
    assert_eq!(*child.inject::<u32>().unwrap(), 20);
    assert_eq!(*root.inject::<u32>().unwrap(), 10);
    assert!(child.inject::<bool>().is_none());
}
#[test]
fn worker_panic_closes_dispatch() {
    let (tx, rx) = mpsc::channel();
    let worker = Worker::spawn((), 2, move || {
        let _ = tx.send(());
    })
    .unwrap();
    worker
        .sender()
        .dispatch(|_, _| panic!("intentional test failure"))
        .unwrap();
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(worker.has_panicked());
    assert_eq!(
        worker.sender().dispatch(|_, _| {}),
        Err(DispatchError::Closed)
    );
}

#[test]
fn dropping_ui_handle_cancels_without_joining_a_busy_worker() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (cancelled_tx, cancelled_rx) = mpsc::channel();
    let worker = Worker::spawn((), 2, || {}).unwrap();
    let sender = worker.sender();
    sender
        .dispatch(move |_, context| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            cancelled_tx.send(context.is_cancelled()).unwrap();
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    drop(worker);
    assert_eq!(sender.dispatch(|_, _| {}), Err(DispatchError::Closed));
    release_tx.send(()).unwrap();
    assert!(cancelled_rx.recv_timeout(Duration::from_secs(2)).unwrap());
}
