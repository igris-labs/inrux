//! Portable multiline editing descriptions; native services remain adapter-owned.
use crate::{InputEvent, Node, NodeKind};
use inrux_core::Emit;
impl Node {
    /// Create a controlled, wrapped multiline text area.
    pub fn text_area(value: String, placeholder: String, event: Emit<String>) -> Self {
        Self::model_text_area(value, placeholder, InputEvent::Plain(event))
    }
    /// Create a multiline editor with coordinator editing acknowledgements.
    pub fn model_text_area(value: String, placeholder: String, event: InputEvent) -> Self {
        Self::new(
            NodeKind::Input {
                multiline: true,
                value: crate::normalize_text(&value, true).into_owned(),
                placeholder: crate::normalize_text(&placeholder, true).into_owned(),
                event,
            },
            vec![],
        )
    }
    /// Explicit single-line spelling of [`Self::input`].
    pub fn text_input(value: String, placeholder: String, event: Emit<String>) -> Self {
        Self::input(value, placeholder, event)
    }
}
