//! Read template values and update models through ordinary fields or tracked handles.
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

/// Compiler-facing reads for direct template bindings.
///
/// A generated call borrows the concrete field once. Reactive handles have an
/// exact receiver implementation; ordinary values use Rust's additional autoref.
#[doc(hidden)]
pub trait ViewValue {
    /// Owned value exposed by this binding.
    type Value;
    /// Borrowed sequence for plain fields, owned sequence for reactive reads.
    type Sequence;
    /// Read a loop source without cloning ordinary collections.
    fn __inrux_view_each(self) -> Result<Self::Sequence, inrux_core::reactive::ReactiveError>;
    /// Read an owned prop value, tracking reactive dependencies where applicable.
    fn __inrux_view_value(self) -> Result<Self::Value, inrux_core::reactive::ReactiveError>
    where
        Self::Value: Clone;
    /// Read display text without cloning an ordinary String before formatting it.
    fn __inrux_view_text(self) -> Result<String, inrux_core::reactive::ReactiveError>
    where
        Self::Value: ToString;
}
impl<'a, T> ViewValue for &&'a T {
    type Value = T;
    type Sequence = &'a T;
    fn __inrux_view_each(self) -> Result<Self::Sequence, inrux_core::reactive::ReactiveError> {
        Ok(*self)
    }
    fn __inrux_view_value(self) -> Result<T, inrux_core::reactive::ReactiveError>
    where
        T: Clone,
    {
        Ok(Clone::clone(*self))
    }
    fn __inrux_view_text(self) -> Result<String, inrux_core::reactive::ReactiveError>
    where
        T: ToString,
    {
        Ok((*self).to_string())
    }
}
impl<T: Clone + Send + Sync + 'static> ViewValue for &Signal<T> {
    type Value = T;
    type Sequence = T;
    fn __inrux_view_each(self) -> Result<T, inrux_core::reactive::ReactiveError> {
        self.__inrux_view_value()
    }
    fn __inrux_view_value(self) -> Result<T, inrux_core::reactive::ReactiveError> {
        Ok(self.get())
    }
    fn __inrux_view_text(self) -> Result<String, inrux_core::reactive::ReactiveError>
    where
        T: ToString,
    {
        Ok(self.get().to_string())
    }
}
impl<T: Clone + PartialEq + Send + Sync + 'static> ViewValue
    for &inrux_core::reactive::Computed<T>
{
    type Value = T;
    type Sequence = T;
    fn __inrux_view_each(self) -> Result<T, inrux_core::reactive::ReactiveError> {
        self.__inrux_view_value()
    }
    fn __inrux_view_value(self) -> Result<T, inrux_core::reactive::ReactiveError> {
        self.get()
    }
    fn __inrux_view_text(self) -> Result<String, inrux_core::reactive::ReactiveError>
    where
        T: ToString,
    {
        self.get().map(|value| value.to_string())
    }
}

#[cfg(test)]
mod view_tests {
    use super::*;
    use inrux_core::reactive::Computed;
    #[test]
    fn ordinary_sequences_are_borrowed_and_text_does_not_require_clone() {
        struct Label;
        impl std::fmt::Display for Label {
            fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                output.write_str("label")
            }
        }
        let label = Label;
        assert_eq!((&label).__inrux_view_text().unwrap(), "label");
        let rows = vec![String::from("row")];
        assert!(std::ptr::eq((&rows).__inrux_view_each().unwrap(), &rows));
    }
    #[test]
    fn concrete_receiver_reads_plain_values_signals_and_computed() {
        let name = String::from("Inrux");
        assert_eq!((&name).__inrux_view_value().unwrap(), "Inrux");
        let count = Signal::new(2_u32);
        let source = count.clone();
        let doubled = Computed::new(move || source.get() * 2);
        assert_eq!((&count).__inrux_view_value().unwrap(), 2);
        assert_eq!((&doubled).__inrux_view_value().unwrap(), 4);
        count.set(3);
        assert_eq!((&doubled).__inrux_view_text().unwrap(), "6");
        assert_eq!(name, "Inrux");
    }
}
