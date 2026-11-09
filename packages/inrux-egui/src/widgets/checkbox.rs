//! Native interaction and accessible state with the shared checkbox appearance.
use crate::widgets::Visual;
use eframe::egui;
use inrux_adapter::controls::ACCENT;
use inrux_components::{Node, NodeKind};
use inrux_core::DispatchError;
/// Paint one controlled checkbox without accepting a toolkit-specific skin.
pub(crate) fn paint(
    node: &Node,
    ui: &mut egui::Ui,
    visual: &Visual<'_>,
) -> Result<(), DispatchError> {
    let NodeKind::Checkbox {
        label,
        kind,
        checked,
        event,
    } = &node.kind
    else {
        return Ok(());
    };
    let response = ui
        .add_enabled_ui(!node.disabled, |ui| {
            ui.interact(visual.rect, visual.path, egui::Sense::click())
        })
        .inner;
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, !node.disabled, *checked, label)
    });
    ui.ctx()
        .data_mut(|data| data.insert_temp(visual.path.with("widget-id"), response.id));
    let focused = response.has_focus() && crate::focus::visible(ui.ctx());
    let parts =
        inrux_adapter::control_parts::toggle(node, visual.style, response.hovered(), focused);
    let (indicator_width, indicator_height) = (parts.width, parts.height);
    let switch = *kind == inrux_components::ToggleKind::Switch;
    ui.ctx().accesskit_node_builder(response.id, |node| {
        if switch {
            node.set_role(egui::accesskit::Role::Switch);
        }
    });
    let to_color = |value: inrux_style::Color| {
        egui::Color32::from_rgba_unmultiplied(value.0, value.1, value.2, value.3)
    };
    let origin = egui::pos2(
        visual.rect.left(),
        visual.rect.center().y - indicator_height / 2.0,
    );
    let square = egui::Rect::from_min_size(origin, egui::vec2(indicator_width, indicator_height));
    ui.painter().rect(
        square,
        parts.radius,
        to_color(parts.fill),
        egui::Stroke::new(parts.border_width, to_color(parts.border)),
        egui::StrokeKind::Inside,
    );
    if switch {
        let (x, y, width, height) = parts.thumb_rect(*checked);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(origin + egui::vec2(x, y), egui::vec2(width, height)),
            parts.thumb_radius,
            to_color(parts.mark),
        );
    } else if *checked {
        ui.painter().add(egui::Shape::line(
            vec![
                origin + egui::vec2(parts.width * 4.0 / 18.0, parts.height * 9.0 / 18.0),
                origin + egui::vec2(parts.width * 8.0 / 18.0, parts.height * 13.0 / 18.0),
                origin + egui::vec2(parts.width * 14.0 / 18.0, parts.height * 5.0 / 18.0),
            ],
            egui::Stroke::new(2.0_f32, to_color(parts.mark)),
        ));
    }
    let font = egui::FontId::new(
        parts.label.font_size.unwrap_or(15.0),
        parts
            .label
            .font_family
            .as_ref()
            .map_or(egui::FontFamily::Proportional, |name| {
                egui::FontFamily::Name(name.clone().into())
            }),
    );
    let ink = parts.label.color.map_or(visual.ink, to_color);
    let galley = crate::typography::layout(ui, label, font.clone(), ink, f32::INFINITY);
    let label_origin = egui::pos2(
        origin.x + indicator_width + parts.gap,
        visual.rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(label_origin, galley.clone(), ink);
    crate::typography::paint(
        ui,
        label,
        &font,
        ink,
        f32::INFINITY,
        egui::pos2(
            origin.x + indicator_width + parts.gap,
            visual.rect.center().y - galley.size().y / 2.0,
        ),
    );
    if response.clicked() && !node.disabled {
        response.request_focus();
    }
    if response.has_focus() && crate::focus::visible(ui.ctx()) && !node.has_focus_decoration() {
        ui.painter().rect_stroke(
            square,
            parts.radius,
            egui::Stroke::new(
                2.0_f32,
                to_color(visual.style.accent_color.unwrap_or(ACCENT)),
            ),
            egui::StrokeKind::Inside,
        );
    }
    if response.clicked() && !node.disabled {
        event.emit(!checked)?;
    }
    Ok(())
}
