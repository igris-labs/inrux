//! Shared text shaping feeds the common Taffy geometry policy.
use crate::widgets::text;
use gpui::Window;
use inrux_adapter::{
    AdapterError,
    layout::{self, LayoutBox},
    services::{
        layout::{LayoutRequest, LayoutService, TaffyLayout},
        text::{TextService, TextSize},
    },
};
use inrux_components::Node;
/// Shape with the same shared font and line height used by the private text binding.
pub(crate) fn compute(node: &Node, width: f32, window: &Window) -> Result<LayoutBox, AdapterError> {
    let text = TextService::new(crate::services::TEXT, |request: layout::TextMeasure<'_>| {
        if request.text.is_empty() {
            return Ok(TextSize {
                width: 0.0,
                height: request.font_size * text::LINE_HEIGHT,
            });
        }
        let shaped = inrux_text::shape(
            request.text,
            request.font_size,
            request.font_family,
            request.max_width,
        );
        let (width, height) = (shaped.width(), shaped.height());
        Ok(TextSize { width, height })
    });
    LayoutService::new(TaffyLayout::new(text)).compute(LayoutRequest {
        root: node,
        width,
        height: Some(window.viewport_size().height.into()),
    })
}
