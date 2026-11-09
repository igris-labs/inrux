//! Native event/focus/accessibility services with Inrux-owned checkbox painting.
use super::text;
use crate::widgets::label::Label;
use inrux_adapter::control_parts::ToggleVisual;
use inrux_components::ToggleKind;
use masonry::{
    accesskit,
    core::*,
    kurbo::{Affine, BezPath, Point, RoundedRect, Size, Stroke},
    peniko::Fill,
    ui_events::{
        keyboard::{Key, NamedKey},
        pointer::{PointerButton, PointerButtonEvent},
    },
    vello::Scene,
    widgets::CheckboxToggled,
};
/// Private retained binding; its checked value remains controlled by the framework model.
pub(crate) struct NativeCheckbox {
    checked: bool,
    kind: ToggleKind,
    visual: ToggleVisual,
    title: String,
    label: WidgetPod<Label>,
    label_origin: Point,
}
/// Construct a labelled control with shared typography.
pub(crate) fn build(
    kind: ToggleKind,
    checked: bool,
    title: &str,
    visual: &ToggleVisual,
) -> NewWidget<NativeCheckbox> {
    NewWidget::new(NativeCheckbox {
        checked,
        kind,
        visual: visual.clone(),
        title: title.into(),
        label_origin: Point::ORIGIN,
        label: NewWidget::new_with_props(
            text::label(title, &visual.label).single_line(),
            text::properties(&visual.label),
        )
        .to_pod(),
    })
}
/// Update the controlled value while preserving native focus and the accessible child.
pub(crate) fn update(
    widget: &mut WidgetMut<'_, NativeCheckbox>,
    kind: ToggleKind,
    checked: bool,
    title: &str,
    visual: &ToggleVisual,
) {
    let previous = &widget.widget.visual;
    let metrics_changed = previous.width != visual.width
        || previous.height != visual.height
        || previous.gap != visual.gap;
    text::update(
        &mut widget.ctx.get_mut(&mut widget.widget.label),
        title,
        &visual.label,
        Some(&previous.label),
    );
    let changed = widget.widget.checked != checked
        || widget.widget.kind != kind
        || widget.widget.visual != *visual
        || widget.widget.title != title;
    if !changed {
        return;
    }
    widget.widget.checked = checked;
    widget.widget.kind = kind;
    widget.widget.visual = visual.clone();
    widget.widget.title = title.into();
    if metrics_changed {
        widget.ctx.request_layout();
    }
    widget.ctx.request_render();
}
impl Widget for NativeCheckbox {
    type Action = CheckboxToggled;
    /// Let Masonry maintain pointer capture and dispatch a controlled boolean action.
    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        if ctx.is_disabled() {
            return;
        }
        match event {
            PointerEvent::Down(PointerButtonEvent {
                button: None | Some(PointerButton::Primary),
                ..
            }) => {
                ctx.capture_pointer();
                ctx.request_focus();
            }
            PointerEvent::Up(PointerButtonEvent {
                button: None | Some(PointerButton::Primary),
                ..
            }) if ctx.is_active() && ctx.is_hovered() => {
                ctx.submit_action::<CheckboxToggled>(CheckboxToggled(!self.checked))
            }
            _ => {}
        }
    }
    /// Space and Enter activate the same controlled action as pointer input.
    fn on_text_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _: &mut PropertiesMut<'_>,
        event: &TextEvent,
    ) {
        if ctx.is_disabled() {
            return;
        }
        if let TextEvent::Keyboard(event) = event
            && event.state.is_up()
            && (matches!(&event.key, Key::Character(value) if value == " ")
                || event.key == Key::Named(NamedKey::Enter))
        {
            ctx.submit_action::<CheckboxToggled>(CheckboxToggled(!self.checked));
        }
    }
    /// Native Tab traversal owns focus admission.
    fn accepts_focus(&self) -> bool {
        true
    }
    /// Assistive activation follows the same controlled event path.
    fn on_access_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _: &mut PropertiesMut<'_>,
        event: &AccessEvent,
    ) {
        if !ctx.is_disabled() && event.action == accesskit::Action::Click {
            ctx.submit_action::<CheckboxToggled>(CheckboxToggled(!self.checked));
        }
    }
    /// Repaint state affordances on native interaction changes.
    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, _: &Update) {
        ctx.request_paint_only();
    }
    /// Keep text in the engine-owned widget tree.
    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.label);
    }
    /// Center the label and square inside the common logical control bounds.
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        _: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        let (indicator_width, indicator_height) = (self.visual.width, self.visual.height);
        let label = ctx.run_layout(
            &mut self.label,
            &BoxConstraints::new(
                Size::ZERO,
                Size::new(
                    (bc.max().width - f64::from(indicator_width + self.visual.gap)).max(0.0),
                    bc.max().height,
                ),
            ),
        );
        let size = bc.constrain(Size::new(
            f64::from(indicator_width + self.visual.gap) + label.width,
            f64::from(indicator_height).max(label.height),
        ));
        self.label_origin = Point::new(
            (indicator_width + self.visual.gap).into(),
            (size.height - label.height) / 2.0,
        );
        ctx.place_child(&mut self.label, self.label_origin);
        size
    }
    /// Preserve fractional label centering with the same transform used for hit testing.
    fn compose(&mut self, ctx: &mut masonry::core::ComposeCtx<'_>) {
        ctx.set_animated_child_scroll_translation(
            &mut self.label,
            self.label_origin - self.label_origin.round(),
        );
    }
    /// Draw the shared square/checkmark instead of toolkit-specific hover/focus chrome.
    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _: &PropertiesRef<'_>, scene: &mut Scene) {
        let (width, height) = (self.visual.width, self.visual.height);
        let y = (ctx.size().height - f64::from(height)) / 2.0;
        let switch = self.kind == ToggleKind::Switch;
        let rect = RoundedRect::new(
            f64::from(self.visual.border_width) / 2.0,
            y + f64::from(self.visual.border_width) / 2.0,
            f64::from(width - self.visual.border_width / 2.0),
            y + f64::from(height - self.visual.border_width / 2.0),
            f64::from(self.visual.radius),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            text::color(self.visual.fill),
            None,
            &rect,
        );
        scene.stroke(
            &Stroke::new(self.visual.border_width.into()),
            Affine::IDENTITY,
            text::color(self.visual.border),
            None,
            &rect,
        );
        if switch {
            let (x, ty, w, h) = self.visual.thumb_rect(self.checked);
            let thumb = masonry::kurbo::Rect::new(
                x.into(),
                y + f64::from(ty),
                f64::from(x + w),
                y + f64::from(ty + h),
            )
            .to_rounded_rect(f64::from(self.visual.thumb_radius));
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                text::color(self.visual.mark),
                None,
                &thumb,
            );
        } else if self.checked {
            let mut path = BezPath::new();
            path.move_to((
                f64::from(width * 4.0 / 18.0),
                y + f64::from(height * 9.0 / 18.0),
            ));
            path.line_to((
                f64::from(width * 8.0 / 18.0),
                y + f64::from(height * 13.0 / 18.0),
            ));
            path.line_to((
                f64::from(width * 14.0 / 18.0),
                y + f64::from(height * 5.0 / 18.0),
            ));
            scene.stroke(
                &Stroke::new(2.0),
                Affine::IDENTITY,
                text::color(self.visual.mark),
                None,
                &path,
            );
        }
    }
    /// Expose the semantic role rather than deriving it from paint or class names.
    fn accessibility_role(&self) -> accesskit::Role {
        if self.kind == ToggleKind::Switch {
            accesskit::Role::Switch
        } else {
            accesskit::Role::CheckBox
        }
    }
    /// Keep label, checked state and activation available to accessibility clients.
    fn accessibility(
        &mut self,
        _: &mut AccessCtx<'_>,
        _: &PropertiesRef<'_>,
        node: &mut accesskit::Node,
    ) {
        // A retained boolean control can switch appearance without replacing its native ID.
        node.set_role(self.accessibility_role());
        node.set_label(self.title.clone());
        node.set_toggled(if self.checked {
            accesskit::Toggled::True
        } else {
            accesskit::Toggled::False
        });
        node.add_action(accesskit::Action::Click);
    }
    /// Preserve native child traversal order.
    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.label.id()])
    }
}
