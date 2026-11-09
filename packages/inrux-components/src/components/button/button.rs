//! Button component construction.
use crate::{Node, NodeKind};
use inrux_core::Emit;

impl Node {
    /// Create an optional event-emitting button.
    pub fn button(text: String, event: Option<Emit<()>>) -> Self {
        let mut node = Self::new(NodeKind::Button(text, event), vec![]);
        node.defaults.rounded = Some(6.0);
        node
    }
}
