//! Revision-bearing input events keep stale acknowledgements from replacing newer edits.
use inrux_core::{DispatchError, Emit};

/// One accepted local editing revision.
#[derive(Clone, Debug)]
pub struct ModelEdit {
    /// Monotonically increasing local revision.
    pub revision: u64,
    /// Complete controlled value after this edit.
    pub value: String,
}
/// Controlled input event transport.
#[derive(Clone)]
pub enum InputEvent {
    /// Plain typed callback for hand-authored nodes and stateless prop forwarding.
    Plain(Emit<String>),
    /// Coordinator acknowledgement paired with the rendered model value.
    Versioned {
        /// Most recent edit applied before constructing this render description.
        applied: u64,
        /// Queue an owned value and its editing revision atomically.
        event: Emit<ModelEdit>,
    },
}
impl InputEvent {
    /// Dispatch with a revision when the generated binding supports acknowledgements.
    pub fn emit(&self, revision: u64, value: String) -> Result<(), DispatchError> {
        match self {
            Self::Plain(event) => event.emit(value),
            Self::Versioned { event, .. } => event.emit(ModelEdit { revision, value }),
        }
    }
    /// Last acknowledged local revision, or no protocol for a plain event.
    pub fn applied(&self) -> Option<u64> {
        match self {
            Self::Plain(_) => None,
            Self::Versioned { applied, .. } => Some(*applied),
        }
    }
}
