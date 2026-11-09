//! Controlled on/off switching shares boolean events, never a backend-specific component.
use crate::{Node, NodeKind, ToggleKind};
use inrux_core::Emit;
impl Node {
    /// Create a labelled switch; pointer and keyboard input request a new controlled value.
    pub fn switch(label: String, checked: bool, event: Emit<bool>) -> Self {
        Self::new(
            NodeKind::Checkbox {
                kind: ToggleKind::Switch,
                label,
                checked,
                event,
            },
            vec![],
        )
        .component_type("Switch")
    }
}
