//! Modal dialog component construction.
use crate::{Node, NodeKind};
use inrux_core::Emit;
impl Node {
    /// Create a controlled modal; Escape/backdrop requests closure through the event.
    pub fn dialog(open: bool, content: Node, on_close: Emit<()>) -> Self {
        Self::new(NodeKind::Dialog { open, on_close }, vec![content])
    }
}
