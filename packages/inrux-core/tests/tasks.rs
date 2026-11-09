//! Tasks wait independently while the coordinator keeps processing application state.
use inrux_core::Worker;
use std::{sync::mpsc, time::Duration};

/// A gated task must not prevent another application update before completion.
#[test]
fn background_task_does_not_block_state_updates_and_delivers_result() {
    let worker = Worker::spawn(0_u32, 8, || {}).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    worker
        .sender()
        .dispatch(move |_, ctx| {
            ctx.spawn_task(
                move |_| {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    Ok::<_, ()>(10)
                },
                move |state, result, ctx| {
                    *state += result.unwrap();
                    ctx.publish(state);
                    done_tx.send(*state).unwrap();
                },
            )
            .unwrap();
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let (edited_tx, edited_rx) = mpsc::channel();
    worker
        .sender()
        .dispatch(move |state, _| {
            *state = 2;
            edited_tx.send(()).unwrap();
        })
        .unwrap();
    edited_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    release_tx.send(()).unwrap();
    assert_eq!(done_rx.recv_timeout(Duration::from_secs(2)).unwrap(), 12);
}

/// Explicit cancellation discards results without mutating coordinator-owned state.
#[test]
fn cancelled_task_never_applies_its_result() {
    let worker = Worker::spawn(0, 8, || {}).unwrap();
    let (handle_tx, handle_rx) = mpsc::channel();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    worker
        .sender()
        .dispatch(move |_, ctx| {
            let handle = ctx
                .spawn_task(
                    move |token| {
                        started_tx.send(()).unwrap();
                        release_rx.recv().unwrap();
                        finished_tx.send(token.is_cancelled()).unwrap();
                        Ok::<_, ()>(1)
                    },
                    |state, _, _| *state = 99,
                )
                .unwrap();
            handle_tx.send(handle).unwrap();
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    handle_rx.recv().unwrap().cancel();
    release_tx.send(()).unwrap();
    assert!(finished_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    let (tx, rx) = mpsc::channel();
    worker
        .sender()
        .dispatch(move |state, _| tx.send(*state).unwrap())
        .unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap(), 0);
}

/// Admission remains bounded even if both execution slots contain uncooperative work.
#[test]
fn saturated_background_queue_reports_full_without_blocking_coordinator() {
    let worker = Worker::spawn(0, 8, || {}).unwrap();
    let mut releases = vec![];
    for _ in 0..2 {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        releases.push(release_tx);
        worker
            .sender()
            .dispatch(move |_, context| {
                context
                    .spawn_task(
                        move |_| {
                            started_tx.send(()).unwrap();
                            release_rx.recv().unwrap();
                            Ok::<_, ()>(())
                        },
                        |_, _, _| {},
                    )
                    .unwrap();
            })
            .unwrap();
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    }
    let (tx, rx) = mpsc::channel();
    worker
        .sender()
        .dispatch(move |_, context| {
            for _ in 0..16 {
                context
                    .spawn_task(|_| Ok::<_, ()>(()), |_, _, _| {})
                    .unwrap();
            }
            tx.send(context.spawn_task(|_| Ok::<_, ()>(()), |_, _, _| {}).err())
                .unwrap();
        })
        .unwrap();
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        Some(inrux_core::tasks::TaskQueueError::Full)
    );
    for release in releases {
        release.send(()).unwrap();
    }
}

/// Task panics become typed results and leave the serialized application worker usable.
#[test]
fn task_panic_isolated_from_application_state() {
    let worker = Worker::spawn(0, 8, || {}).unwrap();
    let (tx, rx) = mpsc::channel();
    worker
        .sender()
        .dispatch(move |_, context| {
            context
                .spawn_task(
                    |_| -> Result<(), ()> { panic!("task fixture") },
                    move |state, result, _| {
                        assert!(matches!(
                            result,
                            Err(inrux_core::tasks::TaskError::Panicked)
                        ));
                        *state = 1;
                        tx.send(()).unwrap();
                    },
                )
                .unwrap();
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(!worker.has_panicked());
}
