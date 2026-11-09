//! Native clipping/input bridge; offset, routing and thumb geometry stay in the shared policy.
use super::element::Element;
use inrux_adapter::services::scroll::{LINE_STEP, ScrollKey, ScrollState, scrollable};
use inrux_style::Style;
use masonry::{
    accesskit,
    core::*,
    kurbo::{Affine, Size, Vec2},
    vello::Scene,
};
use std::{cell::RefCell, rc::Rc};

/// Allocate overlay hit targets once; omitted axes are stashed during layout.
pub(crate) fn bars(
    style: &Style,
    state: Option<&Rc<RefCell<ScrollState>>>,
    parent: WidgetId,
) -> Vec<WidgetPod<ViewportBar>> {
    if !scrollable(style) {
        return vec![];
    }
    let Some(state) = state else {
        return vec![];
    };
    [false, true]
        .into_iter()
        .map(|horizontal| {
            NewWidget::new(ViewportBar {
                colors: style.resolved_scrollbar_colors(),
                horizontal,
                state: state.clone(),
                parent,
            })
            .to_pod()
        })
        .collect()
}
/// Normalize native physical pixels before applying shared logical scrolling.
pub(crate) fn pointer(element: &mut Element, ctx: &mut EventCtx<'_>, event: &PointerEvent) {
    let Some(state) = &element.scroll else {
        return;
    };
    match event {
        PointerEvent::Scroll(event) => {
            let (x, y) = match event.delta {
                ScrollDelta::PixelDelta(delta) => (
                    -delta.x as f32 / ctx.get_scale_factor() as f32,
                    -delta.y as f32 / ctx.get_scale_factor() as f32,
                ),
                ScrollDelta::LineDelta(x, y) => (-x * LINE_STEP, -y * LINE_STEP),
                _ => (0.0, 0.0),
            };
            if state
                .borrow_mut()
                .wheel(x, y, event.state.modifiers.shift())
            {
                ctx.set_handled();
                ctx.request_anim_frame();
                ctx.request_compose();
            }
        }
        PointerEvent::Down(_) if ctx.target() == ctx.widget_id() && scrollable(&element.style) => {
            ctx.request_focus()
        }
        _ => {}
    }
}
/// Only a focused viewport handles navigation; editors retain their own keys.
pub(crate) fn keyboard(element: &mut Element, ctx: &mut EventCtx<'_>, event: &TextEvent) {
    if !ctx.is_focus_target() {
        return;
    }
    if let TextEvent::Keyboard(event) = event
        && event.state.is_down()
        && let keyboard::Key::Named(key) = &event.key
        && let Some(key) = ScrollKey::from_name(&format!("{key:?}"))
        && let Some(state) = &element.scroll
    {
        state.borrow_mut().key(key);
        ctx.set_handled();
        ctx.request_anim_frame();
        ctx.request_compose();
    }
}
/// Translate retained content without remeasuring or rounding fine trackpad movement.
pub(crate) fn compose(element: &mut Element, ctx: &mut ComposeCtx<'_>) {
    let offset = element
        .scroll
        .as_ref()
        .map_or_default(|state| state.borrow().metrics().offset);
    for (index, child) in element.children.iter_mut().enumerate() {
        let geometry = element.geometry.borrow();
        let position = geometry
            .at(&element.path)
            .and_then(|bounds| bounds.children.get(index))
            .map_or(Vec2::ZERO, |bounds| {
                let x = f64::from(bounds.x);
                let y = f64::from(bounds.y);
                // Masonry rounds layout origins to whole logical pixels, independent of display scale.
                Vec2::new(x - x.round(), y - y.round())
            });
        ctx.set_animated_child_scroll_translation(
            child,
            position - Vec2::new(f64::from(offset.x), f64::from(offset.y)),
        );
    }
    if let Some(content) = &mut element.content {
        let padding = if matches!(element.kind, inrux_components::NodeKind::Text(_)) {
            f64::from(element.style.padding.unwrap_or(0.0))
        } else {
            0.0
        };
        ctx.set_animated_child_scroll_translation(
            content,
            Vec2::new(padding - padding.round(), padding - padding.round()),
        );
    }
    if let Some(state) = &element.scroll {
        let geometry = element.geometry.borrow();
        if let Err(error) = state.borrow_mut().notify(geometry.node_at(&element.path)) {
            ctx.submit_action::<inrux_adapter::AdapterError>(error);
        }
        for bar in &mut element.bars {
            let (_, mut bar_ctx) = ctx.get_raw(bar);
            bar_ctx.request_render();
        }
    }
}
/// Private native pointer capture target, separate from framework components.
pub(crate) struct ViewportBar {
    pub(crate) colors: (inrux_style::Color, inrux_style::Color),
    horizontal: bool,
    state: Rc<RefCell<ScrollState>>,
    parent: WidgetId,
}
impl Widget for ViewportBar {
    type Action = ();
    fn register_children(&mut self, _: &mut RegisterCtx<'_>) {}
    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
    fn accessibility_role(&self) -> accesskit::Role {
        accesskit::Role::GenericContainer
    }
    fn accessibility(
        &mut self,
        _: &mut AccessCtx<'_>,
        _: &PropertiesRef<'_>,
        _: &mut accesskit::Node,
    ) {
    }
    /// Edge hover reveals the existing hit target even while its paint is hidden.
    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, event: &Update) {
        if matches!(event, Update::HoveredChanged(_)) {
            ctx.request_render();
        }
    }
    fn layout(
        &mut self,
        _: &mut LayoutCtx<'_>,
        _: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        bc.max()
    }
    fn on_pointer_event(
        &mut self,
        ctx: &mut EventCtx<'_>,
        _: &mut PropertiesMut<'_>,
        event: &PointerEvent,
    ) {
        let mut state = self.state.borrow_mut();
        let Some(bar) = state
            .bars()
            .into_iter()
            .find(|bar| bar.horizontal == self.horizontal)
        else {
            return;
        };
        match event {
            PointerEvent::Down(event)
                if matches!(
                    event.button,
                    None | Some(masonry::ui_events::pointer::PointerButton::Primary)
                ) =>
            {
                let point = ctx.local_position(event.state.position);
                if state.press(point.x as f32 + bar.track.x, point.y as f32 + bar.track.y) {
                    ctx.capture_pointer();
                    ctx.set_focus(self.parent);
                    ctx.set_handled();
                }
            }
            PointerEvent::Move(event) if state.dragging() => {
                let point = ctx.local_position(event.current.position);
                state.drag_to(point.x as f32 + bar.track.x, point.y as f32 + bar.track.y);
                ctx.set_handled();
            }
            PointerEvent::Up(_) | PointerEvent::Cancel(_) => {
                state.release();
                ctx.release_pointer();
                ctx.set_handled();
            }
            _ => return,
        }
        ctx.request_anim_frame();
        ctx.request_layout();
        ctx.request_render();
    }
    /// Captured drags also get a finite expiry pass after releasing the pointer.
    fn on_anim_frame(&mut self, ctx: &mut UpdateCtx<'_>, _: &mut PropertiesMut<'_>, _: u64) {
        ctx.request_render();
        if self.state.borrow().indicator_delay().is_some() {
            ctx.request_anim_frame();
        }
    }
    fn paint(&mut self, ctx: &mut PaintCtx<'_>, _: &PropertiesRef<'_>, scene: &mut Scene) {
        let state = self.state.borrow();
        if !state.indicator_active() && !ctx.has_hovered() {
            return;
        }
        let Some(bar) = state
            .bars()
            .into_iter()
            .find(|bar| bar.horizontal == self.horizontal)
        else {
            return;
        };
        for (index, (rect, color)) in [(bar.track, self.colors.1), (bar.thumb, self.colors.0)]
            .into_iter()
            .enumerate()
        {
            let rect = masonry::kurbo::Rect::new(
                (rect.x - bar.track.x).into(),
                (rect.y - bar.track.y).into(),
                (rect.x - bar.track.x + rect.width).into(),
                (rect.y - bar.track.y + rect.height).into(),
            );
            let inset = if index == 1 {
                2.0_f64.min(rect.width() / 2.0).min(rect.height() / 2.0)
            } else {
                0.0
            };
            let shape = rect
                .inset(-inset)
                .to_rounded_rect(if index == 1 { 3.0 } else { 0.0 });
            scene.fill(
                masonry::vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                super::text::color(color),
                None,
                &shape,
            );
        }
    }
}

/// Bound animation work to the recent-scroll window, repainting only native indicator children.
pub(crate) fn animate(element: &Element, ctx: &mut UpdateCtx<'_>) {
    if element
        .scroll
        .as_ref()
        .is_some_and(|state| state.borrow().indicator_delay().is_some())
    {
        ctx.request_anim_frame();
    }
    ctx.mutate_self_later(|mut widget| {
        let mut element = widget.downcast::<Element>();
        for bar in &mut element.widget.bars {
            element.ctx.get_mut(bar).ctx.request_render();
        }
    });
}
