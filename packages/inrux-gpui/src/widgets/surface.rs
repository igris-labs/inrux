//! Translate resolved Inrux paint properties without using GPUI's public styling as an authoring API.
use gpui::{Div, Stateful, canvas, prelude::*, px, rgba};
use inrux_adapter::surface::Surface;
use inrux_style::{Color, Style};
/// Preserve straight alpha when converting framework color at the native boundary.
pub(crate) fn color(value: Color) -> gpui::Rgba {
    rgba(
        (u32::from(value.0) << 24)
            | (u32::from(value.1) << 16)
            | (u32::from(value.2) << 8)
            | u32::from(value.3),
    )
}
/// Apply explicit surface defaults after native control affordances.
pub(crate) fn apply(mut element: Stateful<Div>, style: &Style) -> Stateful<Div> {
    let surface = Surface::resolve(style);
    element = element
        .rounded(px(surface.radius))
        .text_size(px(style.font_size.unwrap_or(15.0)))
        .line_height(px(style.font_size.unwrap_or(15.0) * 1.2))
        .text_color(color(style.color.unwrap_or(Color(27, 30, 33))));
    if let Some(fill) = surface.background {
        element = element.bg(color(fill));
    }
    if let Some((width, ink)) = surface.border {
        element = border(element, width, ink, surface.radius);
    }
    element
}

/// Draw an inside border without reserving layout space in the native tree.
pub(crate) fn border(element: Stateful<Div>, width: f32, ink: Color, radius: f32) -> Stateful<Div> {
    element.child(
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                window.paint_quad(gpui::quad(
                    bounds,
                    px(radius),
                    gpui::transparent_black(),
                    px(width),
                    color(ink),
                    gpui::BorderStyle::Solid,
                ));
            },
        )
        .absolute()
        .left(px(0.0))
        .top(px(0.0))
        .size_full(),
    )
}

#[cfg(test)]
mod tests {
    /// GPUI receives the same straight-alpha value as the shared compiler and other bindings.
    #[test]
    fn conversion_preserves_alpha() {
        let native = super::color(inrux_style::Color::rgba(20, 40, 200, 128));
        assert!((native.a - 128.0 / 255.0).abs() < 0.00001);
        assert!((native.b - 200.0 / 255.0).abs() < 0.00001);
    }
}
