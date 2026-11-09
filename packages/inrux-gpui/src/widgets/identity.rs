//! Private native IDs preserve the framework's keys and mount generations.
use inrux_components::{Node, NodeKind};
/// Length-prefixed keys avoid path collisions without exposing GPUI IDs to components.
pub(crate) fn child(parent: &str, index: usize, node: &Node) -> String {
    let key = node.key.as_ref().map_or_else(
        || format!("i{index}"),
        |key| format!("k{}:{key}", key.len()),
    );
    let generation = node.generation.as_ref().map_or_else(
        || "none".into(),
        |value| format!("some{}:{value}", value.len()),
    );
    let kind = match node.kind {
        NodeKind::Column => "column",
        NodeKind::Row => "row",
        NodeKind::Text(_) => "text",
        NodeKind::Button(..) => "button",
        NodeKind::Checkbox {
            kind: inrux_components::ToggleKind::Switch,
            ..
        } => "switch",
        NodeKind::Checkbox { .. } => "checkbox",
        NodeKind::Range(ref range) => match range.kind {
            inrux_components::RangeKind::Progress => "progress",
            inrux_components::RangeKind::Meter => "meter",
        },
        NodeKind::Input {
            multiline: true, ..
        } => "textarea",
        NodeKind::Input { .. } => "input",
        NodeKind::Select { .. } => "select",
        NodeKind::Dialog { .. } => "dialog",
        NodeKind::Heartbeat => "heartbeat",
        NodeKind::Empty => "empty",
    };
    format!("{parent}/{key}/g{generation}/{kind}")
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Reordering keeps keyed identities, while remounts and ambiguous-looking keys stay distinct.
    #[test]
    fn keyed_identity_tracks_generation_not_position() {
        let mut node = Node::text("value".into()).key("a/b");
        assert_eq!(child("root", 0, &node), child("root", 9, &node));
        let previous = child("root", 0, &node);
        node.generation = Some(String::new());
        assert_ne!(previous, child("root", 0, &node));
        node.generation = Some("new".into());
        assert_ne!(previous, child("root", 0, &node));
        assert_ne!(
            child("root", 0, &node),
            child("root", 0, &Node::text("value".into()).key("a"))
        );
    }
}
