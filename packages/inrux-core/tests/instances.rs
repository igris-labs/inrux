//! Generational identity and task ownership reject messages after unmount/remount.
use inrux_core::{Instances, Worker};
use std::{sync::mpsc, time::Duration};
/// Reordering handles preserves their own state; slot reuse cannot revive an old handle.
#[test]
fn instances_retain_independent_state_and_reject_stale_handles() {
    let instances = Instances::default();
    let first = instances.mount(1);
    let second = instances.mount(10);
    instances.update(&first, |value| *value += 1);
    let reordered = [second.clone(), first.clone()];
    assert_eq!(instances.with(&reordered[0], |value| *value), Some(10));
    assert_eq!(instances.with(&reordered[1], |value| *value), Some(2));
    assert_eq!(instances.unmount(&first), Some(2));
    let replacement = instances.mount(99);
    assert_ne!(first.to_string(), replacement.to_string());
    assert_eq!(instances.update(&first, |value| *value = 500), None);
    assert_eq!(instances.with(&replacement, |value| *value), Some(99));
    assert_eq!(
        Instances::<i32>::default().with(&second, |value| *value),
        None
    );
}
/// A background operation observes unmount while gated and cannot update the remounted instance.
#[test]
fn unmount_cancels_owned_work_and_remount_uses_a_fresh_generation() {
    let instances = Instances::default();
    let owner = instances.mount(0);
    let worker = Worker::spawn(instances.clone(), 8, || {}).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let task_owner = owner.clone();
    worker
        .sender()
        .dispatch(move |_, ctx| {
            let target = task_owner.clone();
            ctx.spawn_owned_task(
                &task_owner,
                move |token| {
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    finished_tx.send(token.is_cancelled()).unwrap();
                    Ok::<_, ()>(42)
                },
                move |state, result, _| {
                    state.update(&target, |value| *value = result.unwrap());
                },
            )
            .unwrap();
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    instances.unmount(&owner);
    let replacement = instances.mount(7);
    release_tx.send(()).unwrap();
    assert!(finished_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    assert_eq!(instances.with(&replacement, |value| *value), Some(7));
}
/// Handles do not keep the registry's values alive after the final owning root is disposed.
#[test]
fn dropping_registry_invalidates_outstanding_handles() {
    let instances = Instances::default();
    let handle = instances.mount(1);
    drop(instances);
    assert!(!handle.is_mounted());
}

/// Implicit child tasks share the parent pool and inherit cancellation from their mounted context.
#[test]
fn mapped_child_context_delivers_tasks_to_child_and_discards_unmounted_results() {
    let worker = Worker::spawn((), 8, || {}).unwrap();
    let instances = Instances::default();
    let owner = instances.mount(1);
    let child = worker
        .sender()
        .for_instance(instances.clone(), owner.clone());
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    child
        .dispatch(move |state, context| {
            *state = 2;
            context
                .spawn_task(
                    move |token| {
                        started_tx.send(()).unwrap();
                        release_rx.recv().unwrap();
                        finished_tx.send(token.is_cancelled()).unwrap();
                        Ok::<_, ()>(99)
                    },
                    |state, result, _| *state = result.unwrap(),
                )
                .unwrap();
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(instances.unmount(&owner), Some(2));
    let replacement = instances.mount(7);
    release_tx.send(()).unwrap();
    assert!(finished_rx.recv_timeout(Duration::from_secs(2)).unwrap());
    assert_eq!(instances.with(&replacement, |value| *value), Some(7));
}
