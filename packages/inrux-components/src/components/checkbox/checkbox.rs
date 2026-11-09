//! Checkbox component construction.
use crate::{Node, NodeKind};
use inrux_core::Emit;
impl Node {
    /// Create a labelled checkbox with a typed controlled value event.
    pub fn checkbox(label: String, checked: bool, event: Emit<bool>) -> Self {
        Self::new(
            NodeKind::Checkbox {
                kind: crate::ToggleKind::Checkbox,
                label,
                checked,
                event,
            },
            vec![],
        )
    }
}
