//! Retained native range painting and accessibility with shared normalized values.
use inrux_components::{RangeKind, RangeValue};
use inrux_style::{Color, Style};
use masonry::{
    accesskit,
    core::*,
    kurbo::{Affine, Size},
    peniko::Fill,
    vello::Scene,
};
/// Private native leaf; application data and validation remain in the shared description.
pub(crate) struct NativeRange {
    range: RangeValue,
    fraction: f32,
    ink: Color,
    radius: f32,
}
/// Create a native readonly leaf without a frame timer.
pub(crate) fn build(range: &RangeValue, style: &Style) -> NewWidget<NativeRange> {
    NewWidget::new(NativeRange {
        range: range.clone(),
        fraction: range.fraction().unwrap_or_default(),
        ink: style.background.unwrap_or(inrux_adapter::controls::ACCENT),
        radius: style.rounded.unwrap_or(0.0),
    })
}
/// Update data and theme while preserving the native accessibility identity.
pub(crate) fn update(widget: &mut WidgetMut<'_, NativeRange>, range: &RangeValue, style: &Style) {
    widget.widget.range = range.clone();
    widget.widget.fraction = range.fraction().unwrap_or_default();
    widget.widget.ink = style.background.unwrap_or(inrux_adapter::controls::ACCENT);
    widget.widget.radius = style.rounded.unwrap_or(0.0);
    widget.ctx.request_render();
}
impl Widget for NativeRange {
    type Action = ();
    /// This passive leaf has no native children to register.
    fn register_children(&mut self, _: &mut RegisterCtx<'_>) {}
    /// Keep traversal consistent with the leaf descriptor.
    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
    /// Consume the exact bounds assigned by the shared layout service.
    fn layout(
        &mut self,
        _: &mut LayoutCtx<'_>,
        _: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        bc.max()
    }
    /// Paint the fill only; the containing surface owns track and clipping.
    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _: &PropertiesRef<'_>, scene: &mut Scene) {
        if self.fraction > 0.0 {
            let rect = masonry::kurbo::Rect::new(
                0.0,
                0.0,
                ctx.size().width * f64::from(self.fraction),
                ctx.size().height,
            )
            .to_rounded_rect(f64::from(self.radius));
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                super::text::color(self.ink),
                None,
                &rect,
            );
        }
    }
    /// Expose completion and measurement as distinct native roles.
    fn accessibility_role(&self) -> accesskit::Role {
        if self.range.kind == RangeKind::Meter {
            accesskit::Role::Meter
        } else {
            accesskit::Role::ProgressIndicator
        }
    }
    /// Publish the actual application value and bounds, rather than a percentage approximation.
    fn accessibility(
        &mut self,
        _: &mut AccessCtx<'_>,
        _: &PropertiesRef<'_>,
        node: &mut accesskit::Node,
    ) {
        // Masonry initializes roles at mount; a retained range can change its semantic kind.
        node.set_role(self.accessibility_role());
        node.set_label(self.range.label.clone());
        node.set_numeric_value(self.range.value);
        node.set_min_numeric_value(self.range.min);
        node.set_max_numeric_value(self.range.max);
    }
}
