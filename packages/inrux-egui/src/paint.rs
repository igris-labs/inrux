//! Native paint and interaction handling.
use eframe::egui;
use inrux_components::{Node, NodeKind as Kind};

use inrux_adapter::{AdapterError, layout::LayoutBox};
use inrux_style::{Color, Style};
/// Convert the framework sRGB value into the backend color representation.
fn color(c: Color) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(c.0, c.1, c.2, c.3)
}
/// Apply resolved paint overrides while retaining native focus and interaction affordances.
pub(crate) fn style_widgets(ui: &mut egui::Ui, style: &Style, radius: f32) {
    if style.color.is_some() {
        ui.visuals_mut().disabled_alpha = 1.0;
    }
    if let Some(fill) = style.background {
        ui.visuals_mut().text_edit_bg_color = Some(color(fill));
    }
    if let Some(padding) = style.padding {
        ui.spacing_mut().button_padding = egui::vec2(padding, padding);
    }
    let widgets = &mut ui.visuals_mut().widgets;
    for widget in [
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.noninteractive,
    ] {
        if let Some(fill) = style.background {
            widget.weak_bg_fill = color(fill);
            widget.bg_fill = color(fill);
        }
        if let Some(border) = style.border_color {
            widget.bg_stroke.color = color(border);
            widget.bg_stroke.width = style.border_width.unwrap_or(1.0);
        } else if let Some(width) = style.border_width {
            widget.bg_stroke.width = width;
        }
        // Native hover/focus expansion must not exceed the framework's authored control bounds.
        widget.expansion = 0.0;
        widget.corner_radius = egui::CornerRadius::same(radius.clamp(0.0, 255.0) as u8);
    }
}

/// Paint layout geometry and dispatch widget events without evaluating application logic.
pub(crate) fn paint(
    node: &Node,
    layout: &LayoutBox,
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    path: egui::Id,
    error: &mut Option<String>,
    visited: &mut Vec<egui::Id>,
) -> Result<Option<crate::viewport::FocusTarget>, AdapterError> {
    if matches!(node.kind, Kind::Dialog { .. }) {
        return crate::widgets::dialog::paint(node, ui, path, error).map(|()| None);
    }
    let computed = layout;
    let rect = egui::Rect::from_min_size(
        origin + egui::vec2(computed.x, computed.y),
        egui::vec2(computed.width, computed.height),
    );
    if matches!(node.kind, Kind::Empty) {
        return Ok(None);
    }
    let hovered = ui.rect_contains_pointer(rect);
    let widget_id = ui
        .ctx()
        .data(|data| data.get_temp::<egui::Id>(path.with("widget-id")))
        .unwrap_or(path);
    let focused = ui.memory(|memory| memory.has_focus(widget_id))
        && (matches!(node.kind, Kind::Input { .. }) || crate::focus::visible(ui.ctx()));
    let style = node.paint_style(hovered, focused);
    let surface = inrux_adapter::surface::Surface::resolve(&style);
    let radius = surface.radius;
    // Native framed controls own this surface; painting it twice would multiply authored alpha.
    let native_surface = matches!(
        node.kind,
        Kind::Button(..) | Kind::Input { .. } | Kind::Select { .. }
    );
    if let Some(fill) = surface.background.filter(|_| !native_surface) {
        ui.painter().rect_filled(rect, radius, color(fill));
    }
    if let Some((width, border)) = surface.border.filter(|_| !native_surface) {
        ui.painter().rect_stroke(
            rect,
            radius,
            egui::Stroke::new(width, color(border)),
            egui::StrokeKind::Inside,
        );
    }
    let ink = color(style.color.unwrap_or(Color(27, 30, 33)));
    let font = egui::FontId::new(
        style.font_size.unwrap_or(15.0),
        style
            .font_family
            .as_ref()
            .map_or(egui::FontFamily::Proportional, |family| {
                egui::FontFamily::Name(family.clone().into())
            }),
    );
    let visual = crate::widgets::Visual {
        rect,
        path,
        style: &style,
        radius,
        ink,
        font,
    };
    let emit_result = match &node.kind {
        Kind::Text(..) => crate::widgets::text::paint(node, ui, &visual),
        Kind::Button(..) => crate::widgets::button::paint(node, ui, &visual),
        Kind::Input { .. } => crate::widgets::input::paint(node, ui, &visual),
        Kind::Checkbox { .. } => crate::widgets::checkbox::paint(node, ui, &visual),
        Kind::Range(_) => {
            crate::widgets::range::paint(node, ui, &visual);
            Ok(())
        }
        Kind::Select { .. } => crate::widgets::select::paint(node, ui, &visual),
        Kind::Heartbeat => crate::widgets::heartbeat::paint(node, ui, &visual),
        _ => Ok(()),
    };
    if let Err(failure) = emit_result {
        *error = Some(failure.to_string());
    }
    if inrux_adapter::services::scroll::clips(&style) {
        return crate::viewport::paint(node, layout, ui, rect, path, error, visited);
    }
    let descendant = children(node, layout, ui, rect.min, path, error, visited)?;
    let native = ui
        .ctx()
        .data(|data| data.get_temp::<egui::Id>(path.with("widget-id")))
        .unwrap_or(path);
    if matches!(
        node.kind,
        Kind::Input { .. } | Kind::Button(..) | Kind::Checkbox { .. }
    ) && ui.memory(|memory| memory.has_focus(native))
    {
        return Ok(Some(crate::viewport::FocusTarget {
            id: format!("{native:?}"),
            rect: inrux_adapter::services::clipping::ClipRect {
                x: rect.min.x,
                y: rect.min.y,
                width: rect.width(),
                height: rect.height(),
            },
        }));
    }
    Ok(descendant)
}
/// Preserve normal child identity/order while allowing a viewport to translate its content origin.
pub(crate) fn children(
    node: &Node,
    layout: &LayoutBox,
    ui: &mut egui::Ui,
    origin: egui::Pos2,
    path: egui::Id,
    error: &mut Option<String>,
    visited: &mut Vec<egui::Id>,
) -> Result<Option<crate::viewport::FocusTarget>, AdapterError> {
    let mut focus = None;
    for (index, (child, child_layout)) in node.children.iter().zip(&layout.children).enumerate() {
        let child = paint(
            child,
            child_layout,
            ui,
            origin,
            child.key.as_ref().map_or_else(
                || {
                    if child.generation.is_some() {
                        path.with(("index", index, &child.generation))
                    } else {
                        path.with(("index", index))
                    }
                },
                |key| path.with(("key", key, &child.generation)),
            ),
            error,
            visited,
        )?;
        if child.is_some() {
            focus = child;
        }
    }
    Ok(focus)
}
