//! Typed application-state and event contracts.
use crate::Node;
use inrux_core::{Context, Emit, Scope, Sender};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

/// Application state, handlers, and view construction run on the coordinator.
pub trait Component: Clone + Send + Sync + 'static {
    /// Message accepted by the worker-owned component.
    type Action: Send + 'static;
    /// Apply an action on the application worker.
    fn update(&mut self, action: Self::Action, context: &Context<Self>);
    /// Construct an owned render description on the coordinator.
    fn view(&self, events: &Events<Self>, scope: &Scope) -> Node;
}
/// Typed event factory for generated components.
pub struct Events<S> {
    pub(crate) sender: Sender<S>,
    models: Mutex<BTreeMap<String, Arc<AtomicU64>>>,
}
impl<S: Component> Events<S> {
    /// Build event handlers for an application worker.
    pub fn new(sender: Sender<S>) -> Self {
        Self {
            sender,
            models: Mutex::new(BTreeMap::new()),
        }
    }
    /// Infer the emitted payload from the child's typed prop and the ordinary function signature.
    pub fn listener<T, Mode>(&self, handler: impl crate::Handler<S, T, Mode>) -> Emit<T> {
        handler.into_event(self.sender.clone())
    }
    /// Queue an ordinary state-changing function without declaring an action enum.
    pub fn handler(
        &self,
        handler: impl Fn(&mut S, &Context<S>) + Send + Sync + 'static,
    ) -> Emit<()> {
        let sender = self.sender.clone();
        let handler = Arc::new(handler);
        Emit::new(move |()| {
            let handler = handler.clone();
            sender.dispatch(move |state, context| handler(state, context))
        })
    }
    /// Queue a typed payload directly into an ordinary state-changing function.
    pub fn handler_payload<T: Send + 'static>(
        &self,
        handler: impl Fn(&mut S, T, &Context<S>) + Send + Sync + 'static,
    ) -> Emit<T> {
        let sender = self.sender.clone();
        let handler = Arc::new(handler);
        Emit::new(move |value| {
            let handler = handler.clone();
            sender.dispatch(move |state, context| handler(state, value, context))
        })
    }
    /// Queue a payload-free action.
    pub fn action(&self, action: S::Action) -> Emit<()>
    where
        S::Action: Clone + Sync,
    {
        let sender = self.sender.clone();
        let action = Arc::new(action);
        Emit::new(move |()| {
            let action = action.clone();
            sender.dispatch(move |state, context| state.update((*action).clone(), context))
        })
    }
    /// Map a typed emitted payload to an application action.
    pub fn map_action<T: Send + 'static>(
        &self,
        map: impl Fn(T) -> S::Action + Send + Sync + 'static,
    ) -> Emit<T> {
        let sender = self.sender.clone();
        let map = Arc::new(map);
        Emit::new(move |value| {
            let map = map.clone();
            sender.dispatch(move |state, context| state.update(map(value), context))
        })
    }
    /// Apply a model value and its optional update listener atomically on the coordinator.
    pub fn model_with_handler<T: Clone + Send + 'static, Mode>(
        &self,
        setter: impl Fn(&mut S, T, &Context<S>) + Send + Sync + 'static,
        handler: impl crate::Handler<S, T, Mode>,
    ) -> Emit<T> {
        self.model(move |state, value: T, context| {
            let payload = value.clone();
            setter(state, value, context);
            handler.apply(state, payload, context);
        })
    }
    /// Bind a source control identity to acknowledgements captured with each rendered value.
    pub fn model_versioned(
        &self,
        key: &str,
        setter: impl Fn(&mut S, String, &Context<S>) + Send + Sync + 'static,
    ) -> crate::InputEvent {
        let acknowledgement = self
            .models
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(key.to_owned())
            .or_insert_with(|| Arc::new(AtomicU64::new(0)))
            .clone();
        let applied = acknowledgement.load(Ordering::Acquire);
        let sender = self.sender.clone();
        let setter = Arc::new(setter);
        let event = Emit::new(move |edit: crate::ModelEdit| {
            let setter = setter.clone();
            let acknowledgement = acknowledgement.clone();
            sender.dispatch(move |state, context| {
                setter(state, edit.value, context);
                acknowledgement.store(edit.revision, Ordering::Release);
            })
        });
        crate::InputEvent::Versioned { applied, event }
    }
    /// Queue a generated model setter and its optional input action atomically.
    pub fn model<T: Send + 'static>(
        &self,
        setter: impl Fn(&mut S, T, &Context<S>) + Send + Sync + 'static,
    ) -> Emit<T> {
        let sender = self.sender.clone();
        let setter = Arc::new(setter);
        Emit::new(move |value| {
            let setter = setter.clone();
            sender.dispatch(move |state, context| setter(state, value, context))
        })
    }
}
