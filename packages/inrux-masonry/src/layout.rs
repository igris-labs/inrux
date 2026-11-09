//! Shared text measurement and widget placement consume the selected layout service.
use crate::widgets::element::Element;
use inrux_adapter::{
    layout::{self, LayoutBox},
    services::{
        layout::{LayoutRequest, LayoutService, TaffyLayout},
        text::{TextService, TextSize},
    },
};
use inrux_components::Node;
use masonry::{
    core::{BoxConstraints, LayoutCtx},
    kurbo::{Point, Size},
};
use std::sync::Arc;
/// UI-thread-only layout state shared by wrappers in the engine-owned widget tree.
pub(crate) struct LayoutState {
    pub(crate) descriptor: Arc<Node>,
    boxes: LayoutBox,
    computed_for: Option<Size>,
    measurements: crate::text_measure::Measurements,
    pub(crate) viewport: Option<inrux_adapter::ViewportState>,
    #[cfg(test)]
    pub(crate) computations: usize,
    #[cfg(test)]
    pub(crate) native_layouts: usize,
}
impl LayoutState {
    /// Store one owned description for the whole tree rather than cloning each subtree per widget.
    pub(crate) fn new(descriptor: Arc<Node>) -> Self {
        Self {
            descriptor,
            boxes: LayoutBox::default(),
            computed_for: None,
            measurements: Default::default(),
            viewport: None,
            #[cfg(test)]
            computations: 0,
            #[cfg(test)]
            native_layouts: 0,
        }
    }
    /// A new publication invalidates geometry; native hover/focus/scroll do not replace the descriptor.
    pub(crate) fn replace_descriptor(&mut self, descriptor: Arc<Node>) {
        self.descriptor = descriptor;
        self.computed_for = None;
    }
    pub(crate) fn node_at(&self, path: &[usize]) -> &Node {
        let mut node = self.descriptor.as_ref();
        for index in path {
            node = &node.children[*index];
        }
        node
    }
    /// Find geometry using the same logical child order as the framework description.
    pub(crate) fn at(&self, path: &[usize]) -> Option<&LayoutBox> {
        let mut layout = &self.boxes;
        for index in path {
            layout = layout.children.get(*index)?;
        }
        Some(layout)
    }
}
/// Measure via the shared text provider once at the root, then place the retained native children.
pub(crate) fn layout(element: &mut Element, ctx: &mut LayoutCtx<'_>, bc: &BoxConstraints) -> Size {
    #[cfg(test)]
    {
        element.geometry.borrow_mut().native_layouts += 1;
    }
    if element.path.is_empty() && element.geometry.borrow().computed_for != Some(bc.max()) {
        #[cfg(test)]
        {
            element.geometry.borrow_mut().computations += 1;
        }
        if let Some(viewport) = &element.geometry.borrow().viewport {
            viewport.set(bc.max().width as f32, bc.max().height as f32);
        }
        let descriptor = element.geometry.borrow().descriptor.clone();
        let previous = {
            let mut geometry = element.geometry.borrow_mut();
            // Fonts are registered before mounting. Live font registration must clear these metrics too.
            std::mem::take(&mut geometry.measurements)
        };
        let mut measurements = crate::text_measure::Measurements::default();
        let text = TextService::new(crate::services::TEXT, |request: layout::TextMeasure<'_>| {
            if let Some(size) = measurements.get(&request) {
                return Ok(size);
            }
            if let Some(size) = previous.get(&request) {
                measurements.insert(&request, size);
                return Ok(size);
            }
            let measured = inrux_text::shape(
                request.text,
                request.font_size,
                request.font_family,
                request.max_width,
            );
            let size = TextSize {
                width: measured.width(),
                height: measured.height(),
            };
            measurements.insert(&request, size);
            Ok(size)
        });
        let result = LayoutService::new(TaffyLayout::new(text)).compute(LayoutRequest {
            root: &descriptor,
            width: bc.max().width as f32,
            height: bc
                .max()
                .height
                .is_finite()
                .then_some(bc.max().height as f32),
        });
        element.geometry.borrow_mut().measurements = measurements;
        match result {
            Ok(boxes) => {
                let mut geometry = element.geometry.borrow_mut();
                geometry.boxes = boxes;
                geometry.computed_for = Some(bc.max());
            }
            Err(error) => {
                ctx.submit_action::<inrux_adapter::AdapterError>(error);
                return bc.min();
            }
        }
    }
    let (width, height) = element
        .geometry
        .borrow()
        .at(&element.path)
        .map_or((0.0, 0.0), |geometry| (geometry.width, geometry.height));
    if let Some(scroll) = &element.scroll {
        let geometry = element.geometry.borrow();
        if let Some(bounds) = geometry.at(&element.path) {
            let mut state = scroll.borrow_mut();
            let node = geometry.node_at(&element.path);
            state.update(node, bounds);
        }
        ctx.set_clip_path(Size::new(width.into(), height.into()).to_rect());
    } else {
        ctx.clear_clip_path();
    }
    if let Some(content) = &mut element.content {
        let padding = if matches!(element.kind, inrux_components::NodeKind::Text(_)) {
            element.style.padding.unwrap_or(0.0)
        } else {
            0.0
        };
        ctx.run_layout(
            content,
            &BoxConstraints::tight(Size::new(
                (width - padding * 2.0).max(0.0).into(),
                (height - padding * 2.0).max(0.0).into(),
            )),
        );
        ctx.place_child(content, Point::new(padding.into(), padding.into()));
    }
    let mut focused = None;
    let mut placements = Vec::new();
    for (index, child) in element.children.iter_mut().enumerate() {
        let (x, y, width, height) = element
            .geometry
            .borrow()
            .at(&element.path)
            .and_then(|geometry| geometry.children.get(index))
            .map_or((0.0, 0.0, 0.0, 0.0), |geometry| {
                (geometry.x, geometry.y, geometry.width, geometry.height)
            });
        ctx.run_layout(
            child,
            &BoxConstraints::tight(Size::new(width.into(), height.into())),
        );
        let (native, _) = ctx.get_raw(child);
        if let Some((id, mut rect)) = native.focused.clone() {
            rect.x += x;
            rect.y += y;
            focused = Some((id, rect));
        }
        placements.push((x, y));
    }
    if let Some(scroll) = &element.scroll {
        let mut state = scroll.borrow_mut();
        // Reveal only a newly focused descendant, rather than snapping back after every wheel event.
        state.focus(focused.clone());
        focused = focused.map(|(id, _)| {
            (
                id,
                inrux_adapter::services::clipping::ClipRect {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                },
            )
        });
    }
    for (child, (x, y)) in element.children.iter_mut().zip(placements) {
        ctx.place_child(child, Point::new(x.into(), y.into()));
    }
    let own_focus = ctx.is_focus_target() || (element.content.is_some() && ctx.has_focus_target());
    element.focused = focused.or_else(|| {
        own_focus.then(|| {
            (
                format!("{:?}", ctx.widget_id()),
                inrux_adapter::services::clipping::ClipRect {
                    x: 0.0,
                    y: 0.0,
                    width,
                    height,
                },
            )
        })
    });
    if let Some(scroll) = &element.scroll {
        let bars = scroll.borrow().bars();
        for (index, bar) in element.bars.iter_mut().enumerate() {
            let horizontal = index == 1;
            let bounds = bars
                .iter()
                .find(|b| b.horizontal == horizontal)
                .map(|b| b.track);
            ctx.set_stashed(bar, bounds.is_none());
            if let Some(rect) = bounds {
                ctx.run_layout(
                    bar,
                    &BoxConstraints::tight(Size::new(rect.width.into(), rect.height.into())),
                );
                ctx.place_child(bar, Point::new(rect.x.into(), rect.y.into()));
            }
        }
    }
    bc.constrain(Size::new(width.into(), height.into()))
}
