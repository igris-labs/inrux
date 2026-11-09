//! Native clip/input operations around the shared viewport policy.
use eframe::egui;
use inrux_adapter::{
    AdapterError,
    layout::LayoutBox,
    services::{
        clipping::ClipRect,
        scroll::{ScrollKey, ScrollState},
    },
};
use inrux_components::Node;
/// A native focus target, expressed in logical window coordinates for ancestor reveal.
pub(crate) struct FocusTarget {
    pub(crate) id: String,
    pub(crate) rect: ClipRect,
}
/// Paint clipped children, then route remaining input to this viewport (deepest movable viewport wins).
pub(crate) fn paint(
    node: &Node,
    bounds: &LayoutBox,
    ui: &mut egui::Ui,
    rect: egui::Rect,
    path: egui::Id,
    error: &mut Option<String>,
    visited: &mut Vec<egui::Id>,
) -> Result<Option<FocusTarget>, AdapterError> {
    let key = path.with("viewport-state");
    visited.push(key);
    let mut state = ui
        .ctx()
        .data_mut(|data| data.get_temp::<ScrollState>(key))
        .unwrap_or_default();
    state.update(node, bounds);
    let scrollable = inrux_adapter::services::scroll::scrollable(&node.style());
    let response =
        scrollable.then(|| ui.interact(rect, path.with("viewport-focus"), egui::Sense::click()));
    if let Some(response) = &response
        && response.clicked()
    {
        response.request_focus();
    }
    let offset = state.metrics().offset;
    let focus = ui
        .scope_builder(
            egui::UiBuilder::new()
                .id_salt(path.with("viewport"))
                .max_rect(rect),
            |ui| {
                ui.set_clip_rect(ui.clip_rect().intersect(rect));
                crate::paint::children(
                    node,
                    bounds,
                    ui,
                    rect.min - egui::vec2(offset.x, offset.y),
                    path,
                    error,
                    visited,
                )
            },
        )
        .inner?;
    if let Some(target) = &focus {
        if state.focus(Some((
            target.id.clone(),
            ClipRect {
                x: target.rect.x - rect.min.x + offset.x,
                y: target.rect.y - rect.min.y + offset.y,
                width: target.rect.width,
                height: target.rect.height,
            },
        ))) {
            ui.ctx().request_repaint();
        }
    } else {
        state.focus(None);
    }
    if scrollable && ui.rect_contains_pointer(rect) {
        // egui preserves fine trackpad deltas and smooths coarse wheel steps. It
        // already normalizes Shift; applying raw events bypassed this native behavior.
        let delta = ui.input(|input| input.smooth_scroll_delta);
        if state.wheel(-delta.x, -delta.y, false) {
            ui.input_mut(|input| input.smooth_scroll_delta = egui::Vec2::ZERO);
            ui.ctx().request_repaint();
        }
    }
    if response.as_ref().is_some_and(egui::Response::has_focus) {
        for (key, command) in [
            (egui::Key::ArrowUp, ScrollKey::Up),
            (egui::Key::ArrowDown, ScrollKey::Down),
            (egui::Key::ArrowLeft, ScrollKey::Left),
            (egui::Key::ArrowRight, ScrollKey::Right),
            (egui::Key::PageUp, ScrollKey::PageUp),
            (egui::Key::PageDown, ScrollKey::PageDown),
            (egui::Key::Home, ScrollKey::Home),
            (egui::Key::End, ScrollKey::End),
        ] {
            if ui.input_mut(|input| input.consume_key(egui::Modifiers::NONE, key))
                && state.key(command)
            {
                ui.ctx().request_repaint();
            }
        }
        if crate::focus::visible(ui.ctx()) {
            let ink = inrux_adapter::controls::ACCENT;
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgba_unmultiplied(ink.0, ink.1, ink.2, ink.3),
                ),
                egui::StrokeKind::Inside,
            );
        }
    }
    let to_rect = |r: ClipRect| {
        egui::Rect::from_min_size(
            rect.min + egui::vec2(r.x, r.y),
            egui::vec2(r.width, r.height),
        )
    };
    let (thumb, track) = node.style().resolved_scrollbar_colors();
    for bar in state.bars() {
        let response = ui.interact(
            to_rect(bar.track).intersect(ui.clip_rect()),
            path.with(("scrollbar", bar.horizontal)),
            egui::Sense::click_and_drag().difference(egui::Sense::focusable_noninteractive()),
        );
        if let Some(pointer) = response.interact_pointer_pos() {
            let position = pointer - rect.min;
            if response.is_pointer_button_down_on() && !state.dragging() {
                state.press(position.x, position.y);
                ui.memory_mut(|memory| memory.request_focus(path.with("viewport-focus")));
            }
            if state.dragging() && state.drag_to(position.x, position.y) {
                ui.ctx().request_repaint();
            }
        }
        if state.indicator_active() || response.hovered() {
            ui.painter().rect_filled(
                to_rect(bar.track),
                0.0,
                egui::Color32::from_rgba_unmultiplied(track.0, track.1, track.2, track.3),
            );
            ui.painter().rect_filled(
                to_rect(bar.thumb).shrink(
                    2.0_f32
                        .min(bar.thumb.width / 2.0)
                        .min(bar.thumb.height / 2.0),
                ),
                3.0,
                egui::Color32::from_rgba_unmultiplied(thumb.0, thumb.1, thumb.2, thumb.3),
            );
        }
    }
    if let Some(delay) = state.indicator_delay() {
        ui.ctx().request_repaint_after(delay);
    }
    if !ui.input(|input| input.pointer.primary_down()) {
        state.release();
    }
    if state.metrics().offset != offset {
        ui.ctx().request_repaint();
    }
    if let Err(failure) = state.notify(node) {
        *error = Some(failure.to_string());
    }
    ui.ctx().data_mut(|data| data.insert_temp(key, state));
    let focus = focus.or_else(|| {
        response
            .as_ref()
            .filter(|response| response.has_focus())
            .map(|_| FocusTarget {
                id: format!("{path:?}"),
                rect: ClipRect::default(),
            })
    });
    Ok(focus.map(|target| FocusTarget {
        id: target.id,
        rect: ClipRect {
            x: rect.min.x,
            y: rect.min.y,
            width: rect.width(),
            height: rect.height(),
        },
    }))
}
/// Sweep viewport state when a keyed/generation-bearing subtree is removed.
pub(crate) fn sweep(ui: &egui::Ui, root: egui::Id, visited: Vec<egui::Id>) {
    let registry = root.with("viewport-registry");
    ui.ctx().data_mut(|data| {
        for previous in data.get_temp::<Vec<egui::Id>>(registry).unwrap_or_default() {
            if !visited.contains(&previous) {
                data.remove::<ScrollState>(previous);
            }
        }
        data.insert_temp(registry, visited);
    });
}
