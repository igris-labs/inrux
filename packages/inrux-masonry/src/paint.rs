//! Framework surface styling translated to Masonry/Vello without leaking graphics types.
use crate::widgets::{element::Element, text::color};
use masonry::{
    core::PaintCtx,
    kurbo::{Affine, RoundedRect, Stroke},
    peniko::Fill,
    vello::Scene,
};
/// Paint backgrounds and non-layout borders; native leaves retain their own control visuals.
pub(crate) fn paint(element: &Element, ctx: &mut PaintCtx<'_>, scene: &mut Scene) {
    // Native framed leaves paint their own background/border exactly once, including alpha.
    if matches!(
        element.kind,
        inrux_components::NodeKind::Button(..) | inrux_components::NodeKind::Input { .. }
    ) {
        return;
    }
    let focused = ctx.has_focus_target()
        && (matches!(element.kind, inrux_components::NodeKind::Input { .. })
            || element.focus_visibility.visible());
    let style = element.paint_owner.paint_style(ctx.has_hovered(), focused);
    let surface = inrux_adapter::surface::Surface::resolve(&style);
    let bounds = element
        .geometry
        .borrow()
        .at(&element.path)
        .map_or(ctx.size().to_rect(), |bounds| {
            masonry::kurbo::Rect::new(0.0, 0.0, bounds.width.into(), bounds.height.into())
        });
    let rect = RoundedRect::from_rect(bounds, f64::from(surface.radius));
    if let Some(fill) = surface.background {
        scene.fill(Fill::NonZero, Affine::IDENTITY, color(fill), None, &rect);
    }
    if let Some((width, border)) = surface.border {
        // Vello strokes are centered; inset their centerline to keep the border inside.
        let width = f64::from(width).min(bounds.width()).min(bounds.height());
        let rect = RoundedRect::from_rect(
            bounds.inset(-width / 2.0),
            (f64::from(surface.radius) - width / 2.0).max(0.0),
        );
        scene.stroke(
            &Stroke::new(width),
            Affine::IDENTITY,
            color(border),
            None,
            &rect,
        );
    }
}
