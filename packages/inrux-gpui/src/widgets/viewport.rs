//! Native GPUI clipping/events around the shared scroll provider; no public component lives here.
use gpui::{Div, FocusHandle, MouseButton, SharedString, Stateful, canvas, prelude::*, px};
use inrux_adapter::services::{
    clipping::ClipRect,
    scroll::{LINE_STEP, ScrollKey, ScrollState},
};
use std::{cell::RefCell, rc::Rc};
/// Focus geometry passed up to the nearest viewport in unscrolled content coordinates.
pub(crate) struct FocusTarget {
    pub(crate) id: String,
    pub(crate) rect: ClipRect,
}
/// Bind native low-level input/clip services without invoking GPUI's second layout/scroll algorithm.
pub(crate) fn bind(
    mut element: Stateful<Div>,
    state: Rc<RefCell<ScrollState>>,
    handle: FocusHandle,
    origin: (f32, f32),
    id: &str,
    colors: (inrux_style::Color, inrux_style::Color),
) -> Stateful<Div> {
    let wheel = state.clone();
    let keys = state.clone();
    let keyboard = handle.clone();
    let bar_focus = handle.clone();
    element = element
        .overflow_hidden()
        .track_focus(&handle)
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            window.focus(&handle);
        })
        .on_scroll_wheel(move |event, window, cx| {
            let delta = event.delta.pixel_delta(px(LINE_STEP));
            if wheel.borrow_mut().wheel(
                -f32::from(delta.x),
                -f32::from(delta.y),
                event.modifiers.shift,
            ) {
                cx.stop_propagation();
                window.refresh();
            }
        })
        .on_key_down(move |event, window, cx| {
            if keyboard.is_focused(window)
                && let Some(key) = ScrollKey::from_name(&event.keystroke.key)
            {
                cx.stop_propagation();
                if keys.borrow_mut().key(key) {
                    window.refresh();
                }
            }
        });
    let active = state.borrow().indicator_active();
    for bar in state.borrow().bars() {
        let down = state.clone();
        let bar_focus = bar_focus.clone();
        element = element.child(
            gpui::div()
                .id(SharedString::from(format!(
                    "{id}:scrollbar:{}",
                    bar.horizontal
                )))
                .absolute()
                .left(px(bar.track.x))
                .top(px(bar.track.y))
                .w(px(bar.track.width))
                .h(px(bar.track.height))
                .bg(super::surface::color(colors.1))
                .opacity(if active { 1.0 } else { 0.0 })
                .hover(|style| style.opacity(1.0))
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    cx.stop_propagation();
                    window.focus(&bar_focus);
                    down.borrow_mut().press(
                        f32::from(event.position.x) - origin.0,
                        f32::from(event.position.y) - origin.1,
                    );
                    window.refresh();
                })
                .child(
                    gpui::div()
                        .absolute()
                        .left(px(bar.thumb.x - bar.track.x + 2.0))
                        .top(px(bar.thumb.y - bar.track.y + 2.0))
                        .w(px((bar.thumb.width - 4.0).max(0.0)))
                        .h(px((bar.thumb.height - 4.0).max(0.0)))
                        .rounded(px(3.0))
                        .bg(super::surface::color(colors.0)),
                ),
        );
    }
    let drag = state.clone();
    element.child(
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                let state = drag.clone();
                window.on_mouse_event(move |event: &gpui::MouseMoveEvent, phase, window, cx| {
                    if phase == gpui::DispatchPhase::Capture && state.borrow().dragging() {
                        if state.borrow_mut().drag_to(
                            f32::from(event.position.x) - origin.0,
                            f32::from(event.position.y) - origin.1,
                        ) {
                            window.refresh();
                        }
                        cx.stop_propagation();
                    }
                });
                let state = drag.clone();
                window.on_mouse_event(move |event: &gpui::MouseUpEvent, phase, window, cx| {
                    if phase == gpui::DispatchPhase::Capture
                        && event.button == MouseButton::Left
                        && state.borrow().dragging()
                    {
                        state.borrow_mut().release();
                        window.refresh();
                        cx.stop_propagation();
                    }
                });
            },
        )
        .absolute()
        .left(px(0.0))
        .top(px(0.0))
        .size_full(),
    )
}

/// Coalesce expiry wakes and hold only a weak viewport reference after native unmount.
pub(crate) fn schedule_expiry(
    state: &Rc<RefCell<ScrollState>>,
    window: &gpui::Window,
    cx: &gpui::App,
) {
    let Some(delay) = state.borrow_mut().take_indicator_timer() else {
        return;
    };
    let state = Rc::downgrade(state);
    window
        .spawn(cx, async move |cx| {
            cx.background_executor().timer(delay).await;
            if let Some(state) = state.upgrade() {
                state.borrow_mut().finish_indicator_timer();
                let _ = cx.update(|window, _| window.refresh());
            }
        })
        .detach();
}
