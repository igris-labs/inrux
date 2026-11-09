//! One model protocol supports ordinary Rust fields and automatically tracked signals.
use inrux_core::reactive::Signal;
/// Read/write adapter selected from the controlled property's expected Rust type.
pub trait ModelValue<T> {
    /// Obtain the controlled value, tracking dependencies for a reactive signal.
    fn model_value(&self) -> T;
    /// Apply an owned value inside the serialized application command.
    fn set_model_value(&mut self, value: T);
}
impl<T: Clone> ModelValue<T> for T {
    /// Ordinary owned fields need no additional framework state wrapper.
    fn model_value(&self) -> T {
        self.clone()
    }
    /// Replace the field atomically with its accompanying model handler.
    fn set_model_value(&mut self, value: T) {
        *self = value;
    }
}
impl<T: Clone + PartialEq + Send + Sync + 'static> ModelValue<T> for Signal<T> {
    /// Signal reads participate in automatic view/computed dependency tracking.
    fn model_value(&self) -> T {
        self.get()
    }
    /// Notify subscribers only when a signal's value actually changes.
    fn set_model_value(&mut self, value: T) {
        self.set(value);
    }
}
