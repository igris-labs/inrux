//! Native pointer, keyboard, controlled-state and identity conformance.
use super::*;
use gpui::{Context, Render, TestAppContext, point};
use inrux_style::{Size, Style};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
struct Fixture {
    node: Node,
    widgets: Widgets,
}
impl Render for Fixture {
    /// Exercise the production translator with native GPUI hit testing and focus handling.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let boxes = crate::layout::compute(&self.node, 400.0, window).unwrap();
        div().relative().size_full().child(self.widgets.frame(
            &self.node,
            &boxes,
            Rc::default(),
            window,
            cx,
        ))
    }
}
/// Pointer/keyboard activation, keyed reorder and remounts must agree with shared identities.
#[gpui::test]
fn native_controls_route_events_and_retain_keyed_focus(cx: &mut TestAppContext) {
    let activations = Arc::new(AtomicUsize::new(0));
    let observed = activations.clone();
    let first = Node::button(
        "One".into(),
        Some(inrux_core::Emit::new(move |_| {
            observed.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })),
    )
    .key("first")
    .styled(Style::new().width(Size::Px(80.0)));
    let second = Node::button("Two".into(), None)
        .key("second")
        .styled(Style::new().width(Size::Px(80.0)));
    let row = |children| Node::row(children).styled(Style::new().gap(10.0));
    let initial = row(vec![first.clone(), second.clone()]);
    let root_id = identity::child("root", 0, &initial);
    let first_id = identity::child(&root_id, 0, &first);
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node: initial,
        widgets: Widgets::default(),
    });
    cx.simulate_click(point(px(20.0), px(20.0)), gpui::Modifiers::default());
    assert_eq!(activations.load(Ordering::SeqCst), 1);
    assert!(!view.update(cx, |view, _| view.widgets.focus_visibility.get().visible()));
    cx.simulate_keystrokes("space");
    assert!(view.update(cx, |view, _| view.widgets.focus_visibility.get().visible()));
    assert_eq!(activations.load(Ordering::SeqCst), 2);
    let focus = view.update(cx, |view, _| view.widgets.focus[&first_id].clone());
    view.update(cx, |view, cx| {
        view.node = row(vec![second.clone(), first.clone()]);
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(
        view.update(cx, |view, _| view.widgets.focus[&first_id].clone()),
        focus
    );
    cx.update(|window, _| assert!(focus.is_focused(window)));
    let mut remounted = first;
    remounted.generation = Some("next-mount".into());
    view.update(cx, |view, cx| {
        view.node = row(vec![second, remounted]);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(!view.update(cx, |view, _| view.widgets.focus.contains_key(&first_id)));
}
/// Native pointer input forwards controlled booleans, while disabled nodes reject activation.
#[gpui::test]
fn native_checkbox_routes_controlled_values_and_disabled_state(cx: &mut TestAppContext) {
    let (sent, received) = std::sync::mpsc::channel();
    let event = inrux_core::Emit::new(move |value| {
        sent.send(value).unwrap();
        Ok(())
    });
    let node = Node::checkbox("Enabled".into(), false, event.clone())
        .styled(Style::new().width(Size::Px(200.0)));
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node,
        widgets: Widgets::default(),
    });
    cx.simulate_click(point(px(20.0), px(20.0)), gpui::Modifiers::default());
    assert!(received.try_recv().unwrap());
    view.update(cx, |view, cx| {
        view.node = Node::checkbox("Enabled".into(), true, event)
            .styled(Style::new().width(Size::Px(200.0)));
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(20.0)), gpui::Modifiers::default());
    assert!(!received.try_recv().unwrap());
    view.update(cx, |view, cx| {
        view.node.disabled = true;
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(20.0)), gpui::Modifiers::default());
    assert!(matches!(
        received.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
}
/// Focus and editor identity survive reorder; a new generation drops the old native entity.
#[gpui::test]
fn keyed_input_focus_survives_reorder_and_remount_disposes_editor(cx: &mut TestAppContext) {
    cx.update(input::bind_keys);
    let input = Node::input(
        "original".into(),
        "Name".into(),
        inrux_core::Emit::new(|_| Ok(())),
    )
    .key("editor")
    .styled(Style::new().width(Size::Px(240.0)).height(Size::Px(44.0)));
    let label = Node::text("Summary".into()).key("summary");
    let node = Node::column(vec![input.clone(), label.clone()]);
    let root = identity::child("root", 0, &node);
    let id = identity::child(&root, 0, &input);
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node,
        widgets: Widgets::default(),
    });
    cx.simulate_click(point(px(25.0), px(20.0)), gpui::Modifiers::default());
    let weak = view.update(cx, |view, _| view.widgets.inputs[&id].downgrade());
    cx.update(|window, cx| assert!(view.read(cx).widgets.inputs[&id].read(cx).has_focus(window)));
    view.update(cx, |view, cx| {
        view.node = Node::column(vec![label.clone(), input.clone()]);
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|window, cx| {
        let native = &view.read(cx).widgets.inputs[&id];
        assert_eq!(native.entity_id(), weak.entity_id());
        assert!(native.read(cx).has_focus(window));
    });
    let mut remounted = input;
    remounted.generation = Some("next".into());
    view.update(cx, |view, cx| {
        view.node = Node::column(vec![label, remounted]);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(weak.upgrade().is_none());
}

/// Native event routing clips hidden controls and preserves viewport identity after updates.
#[gpui::test]
fn native_viewport_wheel_keyboard_and_clipping(cx: &mut TestAppContext) {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let button = Node::button(
        "Last".into(),
        Some(inrux_core::Emit::new(move |_| {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })),
    )
    .key("last")
    .styled(Style::new().height(Size::Px(40.0)));
    let make = || {
        Node::scroll_area(vec![
            Node::box_container(vec![]).styled(Style::new().height(Size::Px(280.0))),
            button.clone(),
        ])
        .key("viewport")
        .styled(Style::new().width(Size::Px(200.0)).height(Size::Px(100.0)))
    };
    let node = make();
    let id = identity::child("root", 0, &node);
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node,
        widgets: Widgets::default(),
    });
    cx.simulate_click(point(px(20.0), px(300.0)), gpui::Modifiers::default());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    cx.simulate_event(gpui::ScrollWheelEvent {
        position: point(px(20.0), px(50.0)),
        delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(-1000.0))),
        ..Default::default()
    });
    cx.run_until_parked();
    assert_eq!(
        view.update(cx, |view, _| view.widgets.scrolls[&id]
            .borrow()
            .metrics()
            .offset
            .y),
        220.0
    );
    cx.simulate_click(point(px(20.0), px(80.0)), gpui::Modifiers::default());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let state = view.update(cx, |view, _| view.widgets.scrolls[&id].clone());
    view.update(cx, |view, cx| {
        view.node = make();
        cx.notify();
    });
    cx.run_until_parked();
    assert!(Rc::ptr_eq(
        &state,
        &view.update(cx, |view, _| view.widgets.scrolls[&id].clone())
    ));
    cx.simulate_click(point(px(190.0), px(5.0)), gpui::Modifiers::default());
    cx.simulate_keystrokes("home");
    cx.run_until_parked();
    assert_eq!(state.borrow().metrics().offset.y, 0.0);
    let button_id = identity::child(&id, 1, &button);
    cx.update(|window, cx| window.focus(&view.read(cx).widgets.focus[&button_id]));
    cx.run_until_parked();
    assert_eq!(state.borrow().metrics().offset.y, 220.0);
}

/// Native bubbling stops at the deepest movable viewport and resumes at its edge.
#[gpui::test]
fn nested_viewports_route_wheel_to_the_deepest_movable_target(cx: &mut TestAppContext) {
    let content = || Node::box_container(vec![]).styled(Style::new().height(Size::Px(400.0)));
    let inner = Node::scroll_area(vec![content()])
        .key("inner")
        .styled(Style::new().height(Size::Px(100.0)));
    let node = Node::scroll_area(vec![inner.clone(), content()])
        .key("outer")
        .styled(Style::new().height(Size::Px(140.0)).width(Size::Px(200.0)));
    let root_id = identity::child("root", 0, &node);
    let inner_id = identity::child(&root_id, 0, &inner);
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node,
        widgets: Widgets::default(),
    });
    let wheel = |y| gpui::ScrollWheelEvent {
        position: point(px(30.0), px(40.0)),
        delta: gpui::ScrollDelta::Pixels(point(px(0.0), px(y))),
        ..Default::default()
    };
    cx.simulate_event(wheel(-1000.0));
    cx.run_until_parked();
    view.update(cx, |view, _| {
        assert_eq!(
            view.widgets.scrolls[&root_id].borrow().metrics().offset.y,
            0.0
        );
        assert_eq!(
            view.widgets.scrolls[&inner_id].borrow().metrics().offset.y,
            300.0
        );
    });
    cx.simulate_event(wheel(-40.0));
    cx.run_until_parked();
    assert_eq!(
        view.update(cx, |view, _| view.widgets.scrolls[&root_id]
            .borrow()
            .metrics()
            .offset
            .y),
        40.0
    );
}

/// Capture-phase native handlers continue dragging outside the clipped viewport.
#[gpui::test]
fn viewport_scrollbar_captures_pointer_outside_its_bounds(cx: &mut TestAppContext) {
    let node = Node::scroll_area(vec![
        Node::box_container(vec![]).styled(Style::new().height(Size::Px(400.0))),
    ])
    .styled(Style::new().width(Size::Px(200.0)).height(Size::Px(100.0)));
    let id = identity::child("root", 0, &node);
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node,
        widgets: Widgets::default(),
    });
    cx.simulate_mouse_down(
        point(px(195.0), px(10.0)),
        MouseButton::Left,
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    cx.simulate_mouse_move(
        point(px(500.0), px(300.0)),
        MouseButton::Left,
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    cx.simulate_mouse_up(
        point(px(500.0), px(300.0)),
        MouseButton::Left,
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    view.update(cx, |view, _| {
        let state = view.widgets.scrolls[&id].borrow();
        assert_eq!(state.metrics().offset.y, 300.0);
        assert!(!state.dragging());
    });
}

/// Native text shaping under Center accepts padding-exhausted probes and keeps both axes centered.
#[gpui::test]
fn native_center_with_padded_text_uses_valid_content_widths(cx: &mut TestAppContext) {
    for width in [0.0, 100.0, 400.0] {
        let node = Node::center(vec![
            Node::text("Centered on both axes".into()).styled(Style::new().padding(10.0)),
        ])
        .styled(
            Style::new()
                .width(Size::Px(width))
                .height(Size::Px(150.0))
                .padding(12.0),
        );
        let (view, cx) = cx.add_window_view(|_, _| Fixture {
            node,
            widgets: Widgets::default(),
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let boxes = crate::layout::compute(&view.read(cx).node, 400.0, window).unwrap();
            let child = &boxes.children[0];
            // At zero width, padding exhausts the box; successful native shaping is the contract.
            // Independently rounded edges can shift the center by half a logical pixel.
            if width > 0.0 {
                assert!(
                    (child.x * 2.0 + child.width - boxes.width).abs() <= 1.0,
                    "width {width}: {boxes:?}"
                );
                assert!(
                    (child.y * 2.0 + child.height - boxes.height).abs() <= 1.0,
                    "width {width}: {boxes:?}"
                );
            }
        });
    }
}

/// Switch uses actual GPUI hit testing, keyboard activation and the disabled admission gate.
#[gpui::test]
fn native_switch_is_controlled_and_ranges_render_without_focus_targets(cx: &mut TestAppContext) {
    let (sent, received) = std::sync::mpsc::channel();
    let event = inrux_core::Emit::new(move |value| {
        sent.send(value).unwrap();
        Ok(())
    });
    let initial = Node::switch("Sync".into(), false, event.clone())
        .styled(Style::new().width(Size::Px(200.0)));
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node: initial,
        widgets: Widgets::default(),
    });
    cx.simulate_click(point(px(20.0), px(20.0)), gpui::Modifiers::default());
    assert!(received.try_recv().unwrap());
    view.update(cx, |view, cx| {
        view.node =
            Node::switch("Sync".into(), true, event).styled(Style::new().width(Size::Px(200.0)));
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_keystrokes("space");
    assert!(!received.try_recv().unwrap());
    view.update(cx, |view, cx| {
        view.node.disabled = true;
        cx.notify();
    });
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(20.0)), gpui::Modifiers::default());
    assert!(received.try_recv().is_err());
    view.update(cx, |view, cx| {
        view.node = Node::column(vec![
            Node::progress("Work".into(), 25.0, 100.0),
            Node::meter("Temperature".into(), 0.0, -10.0, 30.0),
        ]);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(view.update(cx, |view, _| view.widgets.focus.is_empty()));
}

/// Native hover invalidation and restyling preserve the controlled switch's keyboard identity.
#[gpui::test]
fn authored_part_hover_and_restyle_keep_native_focus(cx: &mut TestAppContext) {
    use inrux_style::{Color, Part, parse_css};
    let node=Node::switch("Sync".into(),false,inrux_core::Emit::new(|_|Ok(())))
        .class("choice",&parse_css(".choice::part(track) {width:48px;height:28px;background:#112233;} .choice::part(thumb) {width:20px;height:20px;} .choice::part(track):hover {background:#778899;}").unwrap())
        .styled(Style::new().width(Size::Px(200.0)));
    crate::capabilities::validate(&node).unwrap();
    let id = identity::child("root", 0, &node);
    let (view, cx) = cx.add_window_view(|_, _| Fixture {
        node,
        widgets: Widgets::default(),
    });
    cx.simulate_mouse_move(
        point(px(20.0), px(20.0)),
        MouseButton::Left,
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    cx.simulate_click(point(px(20.0), px(20.0)), gpui::Modifiers::default());
    cx.run_until_parked();
    let focus = view.update(cx, |v, _| {
        assert!(v.widgets.hovered[&id].get());
        assert_eq!(
            v.node.part_style(Part::Track, true, false).background,
            Some(Color(119, 136, 153))
        );
        v.widgets.focus[&id].clone()
    });
    view.update(cx, |v, cx| {
        v.node = v
            .node
            .clone()
            .styled_part(Part::Track, Style::new().background(Color(4, 5, 6)));
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(focus, view.update(cx, |v, _| v.widgets.focus[&id].clone()));
    cx.simulate_mouse_move(
        point(px(350.0), px(300.0)),
        MouseButton::Left,
        gpui::Modifiers::default(),
    );
    cx.run_until_parked();
    assert!(!view.update(cx, |v, _| v.widgets.hovered[&id].get()));
}
