//! Select component construction.
use crate::{Node, NodeKind};
use inrux_core::Emit;
impl Node {
    /// Create a single-choice control using owned options and typed change events.
    pub fn select(value: String, options: Vec<String>, event: Emit<String>) -> Self {
        Self::new(
            NodeKind::Select {
                value,
                options,
                event,
            },
            vec![],
        )
    }
}
