//! Native button widget behavior.
use crate::paint::style_widgets;
use crate::widgets::Visual;
use eframe::egui;
use inrux_components::{Node, NodeKind as Kind};
use inrux_core::DispatchError;

/// Paint and handle one native button without running application logic on the UI thread.
pub(crate) fn paint(
    node: &Node,
    ui: &mut egui::Ui,
    visual: &Visual<'_>,
) -> Result<(), DispatchError> {
    let Kind::Button(text, event) = &node.kind else {
        return Ok(());
    };
    let rect = visual.rect;
    let path = visual.path;
    let style = visual.style;
    let radius = visual.radius;
    let ink = visual.ink;
    let font = visual.font.clone();
    let mut emit_result = Ok(());
    ui.scope_builder(egui::UiBuilder::new().id_salt(path).max_rect(rect), |ui| {
        style_widgets(ui, style, radius);
        // Authored hover paint stays authoritative; defaults only fill the missing affordance.
        if !node
            .states
            .iter()
            .any(|(state, layer)| *state == inrux_style::State::Hover && layer.background.is_some())
            && let Some(fill) = style.background
        {
            let fill = inrux_adapter::controls::hovered_button_color(fill);
            let fill = egui::Color32::from_rgba_unmultiplied(fill.0, fill.1, fill.2, fill.3);
            let visuals = &mut ui.visuals_mut().widgets.hovered;
            visuals.bg_fill = fill;
            visuals.weak_bg_fill = fill;
        }
        let response = ui
            .add_enabled_ui(!node.disabled, |ui| {
                ui.put(
                    rect,
                    egui::Button::new(crate::typography::layout(
                        ui,
                        text,
                        font.clone(),
                        ink,
                        f32::INFINITY,
                    )),
                )
            })
            .inner;
        let shaped =
            inrux_text::shape(text, font.size, style.font_family.as_deref(), f32::INFINITY);
        crate::typography::paint(
            ui,
            text,
            &font,
            ink,
            f32::INFINITY,
            rect.center() - egui::vec2(shaped.width(), shaped.height()) * 0.5,
        );
        ui.ctx()
            .data_mut(|data| data.insert_temp(path.with("widget-id"), response.id));
        if response.clicked() {
            response.request_focus();
        }
        if response.has_focus() && crate::focus::visible(ui.ctx()) && !node.has_focus_decoration() {
            let accent = style
                .accent_color
                .unwrap_or(inrux_adapter::controls::ACCENT);
            ui.painter().rect_stroke(
                rect,
                radius,
                egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgba_unmultiplied(accent.0, accent.1, accent.2, accent.3),
                ),
                egui::StrokeKind::Inside,
            );
        }
        if response.clicked()
            && let Some(event) = event
        {
            emit_result = event.emit(());
        }
    });
    emit_result
}
