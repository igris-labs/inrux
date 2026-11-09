//! Native heartbeat widget behavior.
use crate::widgets::Visual;
use eframe::egui;
use inrux_components::{Node, NodeKind as Kind};
use inrux_core::DispatchError;
use std::time::Duration;

/// Paint and handle one native heartbeat without running application logic on the UI thread.
pub(crate) fn paint(
    node: &Node,
    ui: &mut egui::Ui,
    visual: &Visual<'_>,
) -> Result<(), DispatchError> {
    let Kind::Heartbeat = &node.kind else {
        return Ok(());
    };
    let rect = visual.rect;
    let ink = visual.ink;
    let time = ui.input(|input| input.time);
    crate::typography::paint(
        ui,
        "UI thread · live animation",
        &visual.font,
        ink,
        f32::INFINITY,
        rect.min,
    );
    let rail = egui::Rect::from_min_size(
        rect.min + egui::vec2(0.0, 32.0),
        egui::vec2(rect.width(), 6.0),
    );
    ui.painter()
        .rect_filled(rail, 3.0, egui::Color32::from_rgb(225, 233, 231));
    let x = ((time * 1.5).sin() as f32 + 1.0) * 0.5 * (rail.width() - 36.0).max(0.0);
    ui.painter().rect_filled(
        egui::Rect::from_min_size(
            rail.min + egui::vec2(x, 0.0),
            egui::vec2(36.0_f32.min(rail.width()), 6.0),
        ),
        3.0,
        egui::Color32::from_rgb(29, 112, 98),
    );
    if ui.is_rect_visible(rect) {
        ui.ctx().request_repaint_after(Duration::from_millis(16));
    }
    Ok(())
}
