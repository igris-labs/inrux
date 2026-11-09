//! Shared shaping and glyph coverage inside GPUI-owned component geometry.
use super::{Failure, text_image};
use gpui::{IntoElement, canvas, point, prelude::*, px};
use inrux_style::{Color, Style};
pub(crate) use inrux_text::LINE_HEIGHT;
/// Paint the shared immutable layout; controls center its logical line box.
pub(crate) fn element(
    text: String,
    style: Style,
    control: bool,
    centered: bool,
    failure: Failure,
) -> impl IntoElement {
    let font_size = style.font_size.unwrap_or(15.0);
    let padding = style.padding.unwrap_or(0.0);
    let ink = style.color.unwrap_or(Color(27, 30, 33));
    canvas(
        move |bounds, _, _| {
            let width = if control || style.white_space == Some(inrux_style::WhiteSpace::Pre) {
                f32::INFINITY
            } else {
                (f32::from(bounds.size.width) - padding * 2.0).max(0.0)
            };
            inrux_text::shape(&text, font_size, style.font_family.as_deref(), width)
        },
        move |bounds, shaped, window, _| {
            let x = if centered {
                (f32::from(bounds.size.width) - shaped.width()) / 2.0
            } else {
                padding
            };
            let y = if control {
                ((f32::from(bounds.size.height) - shaped.height()) / 2.0).max(0.0)
            } else {
                padding
            };
            if let Err(message) = text_image::paint(
                &shaped,
                bounds.origin + point(px(x), px(y)),
                [ink.0, ink.1, ink.2, ink.3],
                window,
            ) {
                *failure.borrow_mut() = Some(inrux_adapter::AdapterError::Native {
                    backend: "gpui",
                    message,
                });
            }
        },
    )
    .absolute()
    .left(px(0.0))
    .top(px(0.0))
    .size_full()
}
