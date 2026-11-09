//! Private GPUI bindings consume shared descriptions and geometry, never define public components.
use super::{checkbox, events, identity, input, surface, text, viewport};
use gpui::{AnyElement, App, FocusHandle, MouseButton, Window, div, prelude::*, px, rgb};
use inrux_adapter::services::{
    clipping::ClipRect,
    scroll::{self, ScrollState},
};
use inrux_adapter::{AdapterError, layout::LayoutBox};
use inrux_components::ScrollOffset;
use inrux_components::{Node, NodeKind};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};
/// Native UI-only state; removal drops handles for disposed controls.
#[derive(Default)]
pub(crate) struct Widgets {
    focus: HashMap<String, FocusHandle>,
    focus_visibility: Rc<Cell<inrux_adapter::FocusVisibility>>,
    visited: HashSet<String>,
    hovered: HashMap<String, Rc<Cell<bool>>>,
    inputs: HashMap<String, gpui::Entity<input::TextInput>>,
    scrolls: HashMap<String, Rc<RefCell<ScrollState>>>,
}
/// A native callback can report a rejected dispatch without borrowing the coordinator.
pub(crate) type Failure = Rc<RefCell<Option<AdapterError>>>;
/// Placement and identity travel together while descending the logical tree.
struct Placement {
    id: String,
    origin: (f32, f32),
    offset: ScrollOffset,
}
impl Widgets {
    /// The host also observes keys when no descendant has acquired native focus yet.
    pub(crate) fn focus_visibility(&self) -> Rc<Cell<inrux_adapter::FocusVisibility>> {
        self.focus_visibility.clone()
    }
    /// Translate a complete frame, sweeping focus for conditionally removed or remounted nodes.
    pub(crate) fn frame(
        &mut self,
        node: &Node,
        geometry: &LayoutBox,
        error: Failure,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        self.visited.clear();
        let root = self.element(
            node,
            geometry,
            Placement {
                id: identity::child("root", 0, node),
                origin: (0.0, 0.0),
                offset: ScrollOffset::default(),
            },
            error,
            window,
            cx,
        );
        self.hovered.retain(|id, _| self.visited.contains(id));
        self.scrolls.retain(|id, _| self.visited.contains(id));
        self.focus.retain(|id, _| self.visited.contains(id));
        self.inputs.retain(|id, _| self.visited.contains(id));
        root.0
    }
    /// Native placement consumes the selected service geometry without recomputing container layout.
    fn element(
        &mut self,
        node: &Node,
        geometry: &LayoutBox,
        placement: Placement,
        error: Failure,
        window: &mut Window,
        cx: &mut App,
    ) -> (AnyElement, Option<viewport::FocusTarget>) {
        let Placement {
            id,
            origin: parent_origin,
            offset: parent_offset,
        } = placement;
        let is_root = self.visited.is_empty();
        self.visited.insert(id.clone());
        let hover = if node.has_hover_styles() {
            Some(self.hovered.entry(id.clone()).or_default().clone())
        } else {
            self.hovered.remove(&id);
            None
        };
        let focused = (self.focus.get(&id).is_some_and(|h| h.is_focused(window))
            && self.focus_visibility.get().visible())
            || self
                .inputs
                .get(&id)
                .is_some_and(|input| input.read(cx).has_focus(window));
        let style = node.paint_style(hover.as_ref().is_some_and(|state| state.get()), focused);
        let clipped = scroll::clips(&style);
        let scrollable = scroll::scrollable(&style);
        let origin = (
            parent_origin.0 + geometry.x - parent_offset.x,
            parent_origin.1 + geometry.y - parent_offset.y,
        );
        let state = if clipped {
            let state = self.scrolls.entry(id.clone()).or_default().clone();
            state.borrow_mut().update(node, geometry);
            Some(state)
        } else {
            self.scrolls.remove(&id);
            None
        };
        let offset = state.as_ref().map_or(ScrollOffset::default(), |state| {
            state.borrow().metrics().offset
        });

        let mut element = div()
            .id(gpui::SharedString::from(id.clone()))
            .absolute()
            .left(px(geometry.x - parent_offset.x))
            .top(px(geometry.y - parent_offset.y))
            .w(px(geometry.width))
            .h(px(geometry.height));
        if let Some(hover_state) = hover.clone() {
            element = element.on_hover(move |over, window, _| {
                if hover_state.replace(*over) != *over {
                    window.refresh();
                }
            });
        }
        if is_root {
            let pointer = self.focus_visibility.clone();
            let keyboard = self.focus_visibility.clone();
            element = element
                .capture_any_mouse_down(move |_, window, _| {
                    let mut visibility = pointer.get();
                    if visibility.pointer() {
                        pointer.set(visibility);
                        window.refresh();
                    }
                })
                .capture_key_down(move |_, window, _| {
                    let mut visibility = keyboard.get();
                    if visibility.keyboard() {
                        keyboard.set(visibility);
                        window.refresh();
                    }
                });
        }
        match &node.kind {
            NodeKind::Text(text) => {
                element = element.child(text::element(
                    text.clone(),
                    style.clone(),
                    false,
                    false,
                    error.clone(),
                ));
            }
            NodeKind::Button(text, _) => {
                element = element.bg(rgb(0xebebeb)).child(text::element(
                    text.clone(),
                    style.clone(),
                    true,
                    true,
                    error.clone(),
                ));
            }
            NodeKind::Checkbox {
                kind,
                checked,
                label,
                ..
            } => {
                element = element.child(checkbox::element(
                    *kind,
                    *checked,
                    label.clone(),
                    inrux_adapter::control_parts::toggle(
                        node,
                        &style,
                        hover.as_ref().is_some_and(|state| state.get()),
                        focused,
                    ),
                    geometry.width,
                    error.clone(),
                ));
            }
            NodeKind::Range(range) => {
                element = element.child(super::range::element(
                    range,
                    &inrux_adapter::control_parts::range(
                        node,
                        &style,
                        hover.as_ref().is_some_and(|state| state.get()),
                        focused,
                    ),
                ));
            }
            NodeKind::Input { .. } => {
                let mut paint_node = node.paint_snapshot();
                paint_node.defaults = inrux_style::Style::new();
                paint_node.base = inrux_style::Style::new();
                paint_node.inline = style.clone();
                paint_node = paint_node.styled_part(
                    inrux_style::Part::Placeholder,
                    inrux_style::Style::new().color(inrux_adapter::control_parts::placeholder(
                        node,
                        hover.as_ref().is_some_and(|state| state.get()),
                        focused,
                    )),
                );
                let native = self
                    .inputs
                    .entry(id.clone())
                    .or_insert_with(|| {
                        cx.new(|cx| input::TextInput::new(&paint_node, error.clone(), cx))
                    })
                    .clone();
                native.update(cx, |input, cx| input.reconcile(&paint_node, cx));
                element = element.child(native);
            }
            _ => {}
        }
        element = surface::apply(element, &style);
        if matches!(node.kind, NodeKind::Button(..))
            && !node.disabled
            && !node.states.iter().any(|(state, layer)| {
                *state == inrux_style::State::Hover && layer.background.is_some()
            })
        {
            let fill = inrux_adapter::controls::hovered_button_color(
                node.style()
                    .background
                    .unwrap_or(inrux_style::Color(235, 235, 235)),
            );
            element = element.hover(move |style| style.bg(surface::color(fill)));
        }
        if matches!(node.kind, NodeKind::Button(..) | NodeKind::Checkbox { .. }) && !node.disabled {
            let handle = self
                .focus
                .entry(id.clone())
                .or_insert_with(|| cx.focus_handle().tab_stop(true))
                .clone();
            let pointer_node = node.clone();
            let keyboard_node = node.clone();
            let pointer_error = error.clone();
            let key_error = error.clone();
            let keyboard_focus = handle.clone();
            element = element
                .track_focus(&handle)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    window.focus(&handle);
                })
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    if let Err(failure) = events::activate(&pointer_node) {
                        *pointer_error.borrow_mut() = Some(failure);
                        window.refresh();
                    }
                })
                .on_key_down(move |event, window, cx| {
                    if keyboard_focus.is_focused(window)
                        && matches!(event.keystroke.key.as_str(), "enter" | "space")
                    {
                        cx.stop_propagation();
                        if let Err(failure) = events::activate(&keyboard_node) {
                            *key_error.borrow_mut() = Some(failure);
                            window.refresh();
                        }
                    }
                });
            if self.focus[&id].is_focused(window)
                && self.focus_visibility.get().visible()
                && !node.has_focus_decoration()
            {
                element = if matches!(node.kind, NodeKind::Checkbox { .. }) {
                    if let NodeKind::Checkbox { .. } = node.kind {
                        checkbox::focus(
                            element,
                            &inrux_adapter::control_parts::toggle(
                                node,
                                &style,
                                hover.as_ref().is_some_and(|state| state.get()),
                                true,
                            ),
                            &style,
                        )
                    } else {
                        element
                    }
                } else {
                    surface::border(
                        element,
                        1.0,
                        style
                            .accent_color
                            .unwrap_or(inrux_adapter::controls::ACCENT),
                        inrux_adapter::surface::Surface::resolve(&node.style()).radius,
                    )
                };
            }
        } else if !scrollable {
            self.focus.remove(&id);
        }
        if matches!(node.kind, NodeKind::Input { .. })
            && !node.has_focus_decoration()
            && self
                .inputs
                .get(&id)
                .is_some_and(|input| input.read(cx).has_focus(window))
        {
            element = surface::border(
                element,
                1.0,
                style
                    .accent_color
                    .unwrap_or(inrux_adapter::controls::ACCENT),
                inrux_adapter::surface::Surface::resolve(&node.style()).radius,
            );
        }
        let own_focus = matches!(
            node.kind,
            NodeKind::Button(..) | NodeKind::Checkbox { .. } | NodeKind::Input { .. }
        ) && (self
            .focus
            .get(&id)
            .is_some_and(|handle| handle.is_focused(window))
            || self
                .inputs
                .get(&id)
                .is_some_and(|input| input.read(cx).has_focus(window)));
        let mut focused = own_focus.then(|| viewport::FocusTarget {
            id: id.clone(),
            rect: ClipRect {
                x: 0.0,
                y: 0.0,
                width: geometry.width,
                height: geometry.height,
            },
        });
        for (index, (child, child_box)) in node.children.iter().zip(&geometry.children).enumerate()
        {
            let (child, focus) = self.element(
                child,
                child_box,
                Placement {
                    id: identity::child(&id, index, child),
                    origin,
                    offset,
                },
                error.clone(),
                window,
                cx,
            );
            element = element.child(child);
            if focus.is_some() {
                focused = focus;
            }
        }
        if let Some(state) = state {
            let target = focused
                .as_ref()
                .map(|target| (target.id.clone(), target.rect));
            if state.borrow_mut().focus(target) {
                window.refresh();
            }
            if let Err(failure) = state.borrow_mut().notify(node) {
                *error.borrow_mut() = Some(failure);
            }
            element = element.overflow_hidden();
            if scrollable {
                let handle = self
                    .focus
                    .entry(id.clone())
                    .or_insert_with(|| cx.focus_handle().tab_stop(true))
                    .clone();
                let focused_viewport = handle.is_focused(window);
                viewport::schedule_expiry(&state, window, cx);
                element = viewport::bind(
                    element,
                    state,
                    handle,
                    origin,
                    &id,
                    style.resolved_scrollbar_colors(),
                );
                if focused_viewport && self.focus_visibility.get().visible() {
                    element = surface::border(element, 1.0, inrux_adapter::controls::ACCENT, 0.0);
                }
            }
            if focused.is_none()
                && self
                    .focus
                    .get(&id)
                    .is_some_and(|handle| handle.is_focused(window))
            {
                focused = Some(viewport::FocusTarget {
                    id: id.clone(),
                    rect: ClipRect::default(),
                });
            }
            return (
                element.into_any_element(),
                focused.map(|target| viewport::FocusTarget {
                    id: target.id,
                    rect: ClipRect {
                        x: geometry.x,
                        y: geometry.y,
                        width: geometry.width,
                        height: geometry.height,
                    },
                }),
            );
        }
        (
            element.into_any_element(),
            focused.map(|mut target| {
                target.rect.x += geometry.x;
                target.rect.y += geometry.y;
                target
            }),
        )
    }
}

#[cfg(test)]
mod tests;
