//! Native precision and invalidation regressions for the viewport composition path.
use crate::widgets::element::{Element, Routes};
use inrux_components::Node;
use inrux_style::Style;
use masonry::{
    kurbo::Size as NativeSize,
    testing::{TestHarness, TestHarnessParams},
};

/// Fine trackpad deltas should translate retained content without layout or fractional jumps.
#[test]
fn viewport_trackpad_motion_preserves_precision_without_layout() {
    for scale in [1.0, 2.0] {
        trackpad_motion(scale);
    }
}
/// Exercise physical-to-logical conversion, both directions and retained focus at each display scale.
fn trackpad_motion(scale: f64) {
    use inrux_style::Size as LogicalSize;
    use masonry::kurbo::{Point, Vec2};
    let observations = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed = observations.clone();
    let node = Node::scroll_area(
        (0..20)
            .map(|i| {
                Node::button(format!("Row {i}"), None)
                    .styled(Style::new().height(LogicalSize::Px(40.0)))
            })
            .collect(),
    )
    .styled(
        Style::new()
            .height(LogicalSize::Px(100.0))
            .width(LogicalSize::Px(200.0)),
    );
    let node = node.on_scroll(inrux_core::Emit::new(move |metrics| {
        observed.lock().unwrap().push(metrics);
        Ok(())
    }));
    let mut routes = Routes::new();
    let tree = Element::build(&node, &mut routes);
    let mut params = TestHarnessParams::default();
    params.window_size = NativeSize::new(600.0, 400.0);
    params.scale_factor = scale;
    let mut harness = TestHarness::create_with(crate::theme::defaults(), tree, params);
    let first = harness.root_widget().children[0].id();
    let target = *routes.iter().find(|(_, owner)| **owner == first).unwrap().0;
    harness.mouse_click_on(target);
    harness.mouse_move((20.0, 60.0));
    observations.lock().unwrap().clear();
    let initial = harness.root_widget().geometry.borrow().native_layouts;
    let origin = harness.get_widget_with_id(first).ctx().window_transform() * Point::ORIGIN;
    for step in 1..=40 {
        let (delta, distance) = if step <= 20 {
            (-0.75, step)
        } else {
            (0.75, 40 - step)
        };
        harness.mouse_wheel(Vec2::new(0.0, delta));
        let expected = f64::from(distance) * 0.75 / scale;
        let root = harness.root_widget();
        assert_eq!(
            root.scroll.as_ref().unwrap().borrow().metrics().offset.y,
            expected as f32
        );
        let reported = observations.lock().unwrap();
        assert_eq!(reported.len(), step as usize);
        assert_eq!(reported.last().unwrap().offset.y, expected as f32);
        drop(reported);
        let position = harness.get_widget_with_id(first).ctx().window_transform() * Point::ORIGIN;
        assert!(
            (origin.y - position.y - expected).abs() < 0.001,
            "step {step}: origin={origin:?}, position={position:?}"
        );
    }
    assert_eq!(
        harness.root_widget().geometry.borrow().native_layouts,
        initial
    );
}

/// Layout commands and removing overflow must synchronize the same native translation as wheel input.
#[test]
fn viewport_commands_and_overflow_removal_reset_content_transform() {
    use inrux_components::{ScrollOffset, ScrollRequest};
    use inrux_style::{Overflow, Size};
    use masonry::kurbo::{Point, Vec2};
    let scene = |overflow| {
        Node::scroll_area(vec![
            Node::box_container(vec![]).styled(Style::new().height(Size::Px(400.0))),
        ])
        .styled(
            Style::new()
                .width(Size::Px(200.0))
                .height(Size::Px(100.0))
                .overflow(overflow),
        )
    };
    let mut routes = Routes::new();
    let node = scene(Overflow::Auto);
    let tree = Element::build(&node, &mut routes);
    let mut harness = TestHarness::create(crate::theme::defaults(), tree);
    let child = harness.root_widget().children[0].id();
    harness.mouse_move((20.0, 30.0));
    harness.mouse_wheel(Vec2::new(0.0, -10.75));
    let translated = harness.get_widget_with_id(child).ctx().window_transform() * Point::ORIGIN;
    assert_eq!(translated.y, -10.75);
    let requested = scene(Overflow::Auto).scroll_to(ScrollRequest {
        revision: 1,
        offset: ScrollOffset { x: 0.0, y: 75.25 },
    });
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &requested, &mut routes));
    assert_eq!(
        harness.get_widget_with_id(child).ctx().window_transform() * Point::ORIGIN,
        Point::new(0.0, -75.25)
    );
    harness.edit_root_widget(|mut root| {
        Element::reconcile(&mut root, &scene(Overflow::Visible), &mut routes)
    });
    assert_eq!(harness.root_widget().children[0].id(), child);
    assert_eq!(
        harness.get_widget_with_id(child).ctx().window_transform() * Point::ORIGIN,
        Point::ORIGIN
    );
}

/// Fractional line heights must survive native container placement as well as wheel movement.
#[test]
fn fractional_text_positions_match_shared_layout() {
    let node = Node::column(vec![
        Node::text("first".into()),
        Node::text("second".into()),
    ])
    .styled(Style::new().font_size(13.0));
    let mut routes = Routes::new();
    let tree = Element::build(&node, &mut routes);
    let harness = TestHarness::create(crate::theme::defaults(), tree);
    let root = harness.root_widget();
    let expected = root.geometry.borrow().at(&[1]).unwrap().y;
    let position = harness
        .get_widget_with_id(root.children[1].id())
        .ctx()
        .window_origin();
    assert!(
        (position.y - f64::from(expected)).abs() < 0.001,
        "native={position:?}, shared={expected}"
    );
}
