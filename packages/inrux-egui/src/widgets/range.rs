//! Read-only range paint/accessibility; shared descriptors own semantics and normalization.
use crate::widgets::Visual;
use eframe::egui;
use inrux_components::{Node, NodeKind, RangeKind};
/// The common surface paints the track; this binding paints only its bounded fill.
pub(crate) fn paint(node: &Node, ui: &mut egui::Ui, visual: &Visual<'_>) {
    let NodeKind::Range(range) = &node.kind else {
        return;
    };
    let response = ui.interact(visual.rect, visual.path, egui::Sense::hover());
    response.widget_info(|| {
        let mut info =
            egui::WidgetInfo::labeled(egui::WidgetType::ProgressIndicator, true, &range.label);
        info.value = Some(range.value);
        info
    });
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(if range.kind == RangeKind::Meter {
            egui::accesskit::Role::Meter
        } else {
            egui::accesskit::Role::ProgressIndicator
        });
        node.set_label(range.label.clone());
        node.set_numeric_value(range.value);
        node.set_min_numeric_value(range.min);
        node.set_max_numeric_value(range.max);
    });
    let fraction = range.fraction().unwrap_or_default();
    if fraction > 0.0 {
        let style =
            inrux_adapter::control_parts::range(node, visual.style, response.hovered(), false);
        let fill = style.background.unwrap();
        let rect = egui::Rect::from_min_size(
            visual.rect.min,
            egui::vec2(visual.rect.width() * fraction, visual.rect.height()),
        );
        ui.painter().rect_filled(
            rect,
            style.rounded.unwrap_or(0.0),
            egui::Color32::from_rgba_unmultiplied(fill.0, fill.1, fill.2, fill.3),
        );
    }
}
