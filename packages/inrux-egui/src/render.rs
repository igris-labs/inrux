//! Frame rendering from owned component descriptions.
use eframe::egui;
use inrux_components::Node;

use inrux_style::Style;

use crate::paint::paint;
use inrux_adapter::{
    AdapterError, layout,
    services::{
        layout::{LayoutRequest, LayoutService, TaffyLayout},
        text::{TextService, TextSize},
    },
};
/// Render a node tree. Also usable in headless egui tests without creating a native window.
pub fn render(mut node: Node, ui: &mut egui::Ui) -> Result<Option<String>, AdapterError> {
    node.inherit(&Style::new());
    render_ref(&node, ui)
}

/// Paint an already inherited render description without reevaluating application code.
pub fn render_ref(node: &Node, ui: &mut egui::Ui) -> Result<Option<String>, AdapterError> {
    crate::focus::observe(ui.ctx());
    if let Err(error) = node.validate_keys() {
        return Ok(Some(error));
    }
    crate::typography::validate_families(node, ui)?;
    let width = ui.available_width();
    let text = TextService::new(crate::services::TEXT, |request: layout::TextMeasure<'_>| {
        let shaped = inrux_text::shape(
            request.text,
            request.font_size,
            request.font_family,
            request.max_width,
        );
        Ok(TextSize {
            width: shaped.width(),
            height: shaped.height(),
        })
    });
    let computed = LayoutService::new(TaffyLayout::new(text)).compute(LayoutRequest {
        root: node,
        width,
        height: Some(ui.available_height()),
    })?;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(computed.width, computed.height),
        egui::Sense::hover(),
    );
    let mut error = None;
    let mut viewports = Vec::new();
    let root_id = ui.id().with(("inrux-root", &node.key, &node.generation));
    paint(
        node,
        &computed,
        ui,
        rect.min,
        root_id,
        &mut error,
        &mut viewports,
    )?;
    crate::viewport::sweep(ui, ui.id(), viewports);
    Ok(error)
}
