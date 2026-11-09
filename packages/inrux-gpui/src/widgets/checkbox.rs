//! Native boolean-control drawing consumes shared geometry/palette, never component-specific state.
use super::{Failure, surface, text};
use gpui::{prelude::*, *};
use inrux_adapter::control_parts::ToggleVisual;
use inrux_components::ToggleKind;
use inrux_style::Style;
/// One native event target serves both semantic boolean control kinds.
pub(crate) fn element(
    kind: ToggleKind,
    checked: bool,
    label: String,
    visual: ToggleVisual,
    width: f32,
    failure: Failure,
) -> impl IntoElement {
    let (indicator_width, indicator_height) = (visual.width, visual.height);
    let label_style = visual.label.clone();
    let gap = visual.gap;
    div()
        .absolute()
        .left(px(0.0))
        .top(px(0.0))
        .size_full()
        .child(
            canvas(
                |_, _, _| (),
                move |bounds, _, window, _| {
                    let origin = bounds.origin
                        + point(px(0.0), (bounds.size.height - px(indicator_height)) / 2.0);
                    window.paint_quad(quad(
                        Bounds::new(origin, size(px(indicator_width), px(indicator_height))),
                        px(visual.radius),
                        surface::color(visual.fill),
                        px(visual.border_width),
                        surface::color(visual.border),
                        BorderStyle::Solid,
                    ));
                    if kind == ToggleKind::Switch {
                        let (x, y, w, h) = visual.thumb_rect(checked);
                        window.paint_quad(quad(
                            Bounds::new(origin + point(px(x), px(y)), size(px(w), px(h))),
                            px(visual.thumb_radius),
                            surface::color(visual.mark),
                            px(0.0),
                            transparent_black(),
                            BorderStyle::Solid,
                        ));
                    } else if checked {
                        let mut path = PathBuilder::stroke(px(2.0));
                        path.move_to(
                            origin
                                + point(
                                    px(visual.width * 4.0 / 18.0),
                                    px(visual.height * 9.0 / 18.0),
                                ),
                        );
                        path.line_to(
                            origin
                                + point(
                                    px(visual.width * 8.0 / 18.0),
                                    px(visual.height * 13.0 / 18.0),
                                ),
                        );
                        path.line_to(
                            origin
                                + point(
                                    px(visual.width * 14.0 / 18.0),
                                    px(visual.height * 5.0 / 18.0),
                                ),
                        );
                        if let Ok(path) = path.build() {
                            window.paint_path(path, surface::color(visual.mark));
                        }
                    }
                },
            )
            .absolute()
            .left(px(0.0))
            .top(px(0.0))
            .size_full(),
        )
        .child(
            div()
                .absolute()
                .left(px(indicator_width + gap))
                .w(px((width - indicator_width - gap).max(0.0)))
                .h_full()
                .child(text::element(label, label_style, true, false, failure)),
        )
}
/// Outline only the indicator when keyboard focus is visible.
pub(crate) fn focus(element: Stateful<Div>, visual: &ToggleVisual, style: &Style) -> Stateful<Div> {
    let (width, height) = (visual.width, visual.height);
    let radius = visual.radius;
    let ink = style
        .accent_color
        .unwrap_or(inrux_adapter::controls::ACCENT);
    element.child(
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let origin =
                    bounds.origin + point(px(0.0), (bounds.size.height - px(height)) / 2.0);
                window.paint_quad(quad(
                    Bounds::new(origin, size(px(width), px(height))),
                    px(radius),
                    transparent_black(),
                    px(2.0),
                    surface::color(ink),
                    BorderStyle::Solid,
                ));
            },
        )
        .absolute()
        .left(px(0.0))
        .top(px(0.0))
        .size_full(),
    )
}
