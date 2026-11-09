//! Explicit capability negotiation prevents silently changing a component when switching engines.
use crate::AdapterError;
use inrux_components::{Node, NodeKind};
/// Framework primitive kinds, independent of any native widget representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Primitive {
    /// Vertical layout.
    Column,
    /// Horizontal layout.
    Row,
    /// Text display.
    Text,
    /// Activatable button.
    Button,
    /// Controlled text editing.
    Input,
    /// Wrapped multiline text editing.
    TextArea,
    /// Boolean control.
    Checkbox,
    /// Controlled on/off switch.
    Switch,
    /// Determinate completion indicator.
    Progress,
    /// Bounded read-only measurement.
    Meter,
    /// Single-choice popup.
    Select,
    /// Modal interaction.
    Dialog,
    /// Prototype animation indicator.
    Heartbeat,
    /// Empty description.
    Empty,
}
impl Primitive {
    /// Identify a descriptor without inspecting backend widget types.
    pub fn of(node: &Node) -> Self {
        match node.kind {
            NodeKind::Column => Self::Column,
            NodeKind::Row => Self::Row,
            NodeKind::Text(_) => Self::Text,
            NodeKind::Button(..) => Self::Button,
            NodeKind::Input {
                multiline: true, ..
            } => Self::TextArea,
            NodeKind::Input { .. } => Self::Input,
            NodeKind::Checkbox {
                kind: inrux_components::ToggleKind::Switch,
                ..
            } => Self::Switch,
            NodeKind::Checkbox { .. } => Self::Checkbox,
            NodeKind::Range(ref range) => match range.kind {
                inrux_components::RangeKind::Progress => Self::Progress,
                inrux_components::RangeKind::Meter => Self::Meter,
            },
            NodeKind::Select { .. } => Self::Select,
            NodeKind::Dialog { .. } => Self::Dialog,
            NodeKind::Heartbeat => Self::Heartbeat,
            NodeKind::Empty => Self::Empty,
        }
    }
}
/// A backend's advertised primitive support. Property-level validation remains backend-owned.
#[derive(Clone, Copy, Debug)]
pub struct Capabilities {
    /// Backend identifier used in diagnostics.
    pub name: &'static str,
    /// Controls/layouts implemented by this adapter.
    pub primitives: &'static [Primitive],
    /// Whether authored hover/focus/disabled paint layers are supported.
    pub interaction_styles: bool,
}
impl Capabilities {
    /// Check a whole published tree before a backend starts translating it.
    pub fn validate(&self, node: &Node) -> Result<(), AdapterError> {
        node.validate_keys().map_err(AdapterError::Application)?;
        self.validate_node(node)
    }
    /// Descend once after the shared identity validation has completed.
    fn validate_node(&self, node: &Node) -> Result<(), AdapterError> {
        let kind = Primitive::of(node);
        if !self.primitives.contains(&kind) {
            return Err(AdapterError::Unsupported {
                backend: self.name,
                feature: format!("{kind:?} (key: {:?})", node.key),
            });
        }
        if !self.interaction_styles
            && (!node.states.is_empty()
                || node
                    .part_rules
                    .iter()
                    .any(|(_, state, _)| *state != inrux_style::State::Base))
        {
            return Err(AdapterError::Unsupported {
                backend: self.name,
                feature: "interaction stylesheet rules".into(),
            });
        }
        for child in &node.children {
            self.validate_node(child)?;
        }
        Ok(())
    }
}
