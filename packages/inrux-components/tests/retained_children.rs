//! Stateful children run on one coordinator, retaining state across prop changes and keyed reordering.
use inrux_components::{
    Component, ComponentHost, Events, Node, NodeKind, component_scope, render_child,
};
use inrux_core::{Context, Scope, Worker};
use std::{sync::mpsc, time::Duration};
#[derive(Clone)]
struct Parent {
    keys: Vec<u32>,
    label: String,
}
#[derive(Clone)]
struct Child {
    label: String,
    count: u32,
}
impl Component for Child {
    type Action = ();
    /// Ordinary handlers replace reducer actions in this specimen.
    fn update(&mut self, _: (), _: &Context<Self>) {}
    /// Display retained state and dispatch a child-typed function on the parent coordinator.
    fn view(&self, events: &Events<Self>, _: &Scope) -> Node {
        Node::button(
            format!("{}:{}", self.label, self.count),
            Some(events.handler(|state, _| state.count += 1)),
        )
    }
}
impl Component for Parent {
    type Action = ();
    /// The test mutates parent props/order through typed queue closures.
    fn update(&mut self, _: (), _: &Context<Self>) {}
    /// Keyed scope identity, not list position, determines retained state.
    fn view(&self, events: &Events<Self>, scope: &Scope) -> Node {
        Node::column(
            self.keys
                .iter()
                .map(|key| {
                    let scope = component_scope(scope, key);
                    let label = format!("{}-{key}", self.label);
                    let initial = label.clone();
                    render_child(
                        events,
                        &scope,
                        || Child {
                            label: initial,
                            count: 0,
                        },
                        |child| child.label = label,
                    )
                })
                .collect(),
        )
    }
}
/// Prop updates preserve local state; reordered children keep their identity; remounts start fresh.
#[test]
fn children_retain_state_and_reject_old_events_after_unmount() {
    let worker = Worker::spawn(
        Parent {
            keys: vec![1, 2],
            label: "old".into(),
        },
        16,
        || {},
    )
    .unwrap();
    let host = ComponentHost::default();
    let events = Events::new(worker.sender());
    let (tx, rx) = mpsc::channel();
    worker.observe(move |state| {
        tx.send(host.render(state, &events, &Scope::default()))
            .unwrap();
    });
    worker.sender().dispatch(|_, _| {}).unwrap();
    let first = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let first_key = first.children[0].key.clone();
    let NodeKind::Button(_, Some(increment)) = &first.children[0].kind else {
        panic!("missing button")
    };
    increment.emit(()).unwrap();
    let updated = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(matches!(&updated.children[0].kind, NodeKind::Button(label, _) if label == "old-1:1"));
    worker
        .sender()
        .dispatch(|state, _| {
            state.label = "new".into();
            state.keys.reverse();
        })
        .unwrap();
    let reordered = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(reordered.children[1].key, first_key);
    assert!(
        matches!(&reordered.children[1].kind, NodeKind::Button(label, _) if label == "new-1:1")
    );
    worker
        .sender()
        .dispatch(|state, _| state.keys.retain(|key| *key != 1))
        .unwrap();
    rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_eq!(increment.emit(()), Err(inrux_core::DispatchError::Closed));
    worker
        .sender()
        .dispatch(|state, _| state.keys.push(1))
        .unwrap();
    let remounted = rx.recv_timeout(Duration::from_secs(2)).unwrap();
    assert_ne!(remounted.children[1].key, first_key);
    assert!(
        matches!(&remounted.children[1].kind, NodeKind::Button(label, _) if label == "new-1:0")
    );
}
