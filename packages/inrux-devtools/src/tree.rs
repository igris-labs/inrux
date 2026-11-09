//! Inspect owned render descriptors without reevaluating components or invoking handlers.
use eframe::egui;
use inrux_components::{Node, NodeKind};

/// Count rendered descriptions, including currently excluded and modal nodes.
pub fn node_count(node: &Node) -> usize {
    1 + node.children.iter().map(node_count).sum::<usize>()
}
/// Display primitive kinds, stable keys, class names, and resolved style layers.
pub fn inspect(node: &Node, ui: &mut egui::Ui) {
    ui.label(format!("{} render nodes", node_count(node)));
    show_node(node, ui, "root");
}
/// Keep inspection IDs distinct from application widget IDs and sibling display labels.
fn show_node(node: &Node, ui: &mut egui::Ui, path: &str) {
    let kind = match node.kind {
        NodeKind::Column => "Column",
        NodeKind::Row => "Row",
        NodeKind::Text(_) => "Text",
        NodeKind::Button(..) => "Button",
        NodeKind::Input {
            multiline: true, ..
        } => "TextArea",
        NodeKind::Input { .. } => "Input",
        NodeKind::Checkbox {
            kind: inrux_components::ToggleKind::Switch,
            ..
        } => "Switch",
        NodeKind::Checkbox { .. } => "Checkbox",
        NodeKind::Range(ref range) => match range.kind {
            inrux_components::RangeKind::Progress => "Progress",
            inrux_components::RangeKind::Meter => "Meter",
        },
        NodeKind::Select { .. } => "Select",
        NodeKind::Dialog { .. } => "Dialog",
        NodeKind::Heartbeat => "Heartbeat",
        NodeKind::Empty => "Empty",
    };
    egui::CollapsingHeader::new(format!("{kind} {}", node.key.as_deref().unwrap_or("")))
        .id_salt(path)
        .show(ui, |ui| {
            ui.label(format!("Classes: {}", node.classes.join(" ")));
            ui.label(format!("Disabled: {}", node.disabled));
            ui.monospace(format!("{:#?}", node.style()));
            for (index, child) in node.children.iter().enumerate() {
                show_node(child, ui, &format!("{path}/{index}"));
            }
        });
}
