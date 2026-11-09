//! Input component construction.
use crate::{Node, NodeKind};
use inrux_core::Emit;

impl Node {
    /// Create a controlled single-line text input.
    pub fn input(value: String, placeholder: String, event: Emit<String>) -> Self {
        Self::new(
            NodeKind::Input {
                multiline: false,
                value: crate::normalize_text(&value, false).into_owned(),
                placeholder: crate::normalize_text(&placeholder, false).into_owned(),
                event: crate::InputEvent::Plain(event),
            },
            vec![],
        )
    }
    /// Create a controlled input with coordinator editing acknowledgements.
    pub fn model_input(value: String, placeholder: String, event: crate::InputEvent) -> Self {
        Self::new(
            NodeKind::Input {
                multiline: false,
                value: crate::normalize_text(&value, false).into_owned(),
                placeholder: crate::normalize_text(&placeholder, false).into_owned(),
                event,
            },
            vec![],
        )
    }
}
