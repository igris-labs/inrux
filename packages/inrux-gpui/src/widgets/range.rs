//! GPUI paints only a normalized fill inside the framework-owned track surface.
use gpui::{canvas, prelude::*, *};
use inrux_components::RangeValue;
use inrux_style::Style;
/// Shared validation owns range semantics; this native binding never schedules animation.
pub(crate) fn element(range: &RangeValue, style: &Style) -> impl IntoElement {
    let fraction = range.fraction().unwrap_or_default();
    let ink = super::surface::color(style.background.unwrap_or(inrux_adapter::controls::ACCENT));
    let radius = style.rounded.unwrap_or(0.0);
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if fraction > 0.0 {
                window.paint_quad(quad(
                    Bounds::new(
                        bounds.origin,
                        size(bounds.size.width * fraction, bounds.size.height),
                    ),
                    px(radius),
                    ink,
                    px(0.0),
                    transparent_black(),
                    BorderStyle::Solid,
                ));
            }
        },
    )
    .absolute()
    .left(px(0.0))
    .top(px(0.0))
    .size_full()
}
