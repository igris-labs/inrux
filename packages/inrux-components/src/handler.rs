//! Infer payload-free and payload-bearing ordinary Rust handlers through disjoint marker types.
use crate::Component;
use inrux_core::{Context, Emit, Sender};
use std::sync::Arc;
/// Signature marker for `fn(&mut State, &Context<State>)`.
pub struct NoPayload;
/// Signature marker for `fn(&mut State, Payload, &Context<State>)`.
pub struct WithPayload;
/// Typed ordinary function accepted by a generated component event.
pub trait Handler<S, T, Mode>: Send + Sync + 'static {
    /// Adapt this handler to the application's nonblocking bounded event queue.
    fn into_event(self, sender: Sender<S>) -> Emit<T>;
    /// Invoke inside an existing coordinator command, for atomic model setters and their handlers.
    fn apply(&self, state: &mut S, value: T, context: &Context<S>);
}
impl<S: Component, T: Send + 'static, F> Handler<S, T, NoPayload> for F
where
    F: Fn(&mut S, &Context<S>) + Send + Sync + 'static,
{
    /// Allow an unused payload to be omitted from ordinary Rust handler signatures.
    fn into_event(self, sender: Sender<S>) -> Emit<T> {
        let handler = Arc::new(self);
        Emit::new(move |value: T| {
            let handler = handler.clone();
            sender.dispatch(move |state, context| {
                drop(value);
                handler(state, context);
            })
        })
    }
    /// Ignore an unused payload while preserving the setter/handler command boundary.
    fn apply(&self, state: &mut S, _: T, context: &Context<S>) {
        self(state, context);
    }
}
impl<S: Component, T: Send + 'static, F> Handler<S, T, WithPayload> for F
where
    F: Fn(&mut S, T, &Context<S>) + Send + Sync + 'static,
{
    /// Carry a typed child payload to the coordinator before invoking application code.
    fn into_event(self, sender: Sender<S>) -> Emit<T> {
        let handler = Arc::new(self);
        Emit::new(move |value| {
            let handler = handler.clone();
            sender.dispatch(move |state, context| handler(state, value, context))
        })
    }
    /// Deliver the new model value after its assignment in the same command.
    fn apply(&self, state: &mut S, value: T, context: &Context<S>) {
        self(state, value, context);
    }
}
