//! Native text widget behavior.
use crate::widgets::Visual;
use eframe::egui;
use inrux_components::{Node, NodeKind as Kind};
use inrux_core::DispatchError;

/// Paint and handle one native text without running application logic on the UI thread.
pub(crate) fn paint(
    node: &Node,
    ui: &mut egui::Ui,
    visual: &Visual<'_>,
) -> Result<(), DispatchError> {
    let Kind::Text(text) = &node.kind else {
        return Ok(());
    };
    let rect = visual.rect;
    let style = visual.style;
    let ink = visual.ink;
    let font = visual.font.clone();
    let content = rect.shrink(style.padding.unwrap_or(0.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt(visual.path)
            .max_rect(content)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            let width = if style.white_space == Some(inrux_style::WhiteSpace::Pre) {
                f32::INFINITY
            } else {
                content.width()
            };
            let galley = crate::typography::layout(ui, text, font.clone(), ink, width);
            let response = ui.add(egui::Label::new(galley).selectable(true));
            crate::typography::paint(ui, text, &font, ink, width, response.rect.min);
        },
    );
    Ok(())
}
