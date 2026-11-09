//! Native dashboard navigation work and retained identity, independent of GPU timing.
use crate::{
    theme,
    widgets::element::{Element, Routes},
};
use inrux_components::Node;
use inrux_style::{Color, Size, Style, parse_css};
use masonry::{kurbo::Size as NativeSize, testing::TestHarness};
use std::time::Instant;
/// Model a persistent sidebar and changing documentation body using real native controls and text.
fn scene(page: usize) -> Node {
    let rules =
        parse_css(".nav {background:#fafafa;color:#262626;} .nav:hover {background:#dddddd;}")
            .unwrap();
    let sidebar = Node::column(
        (0..24)
            .map(|i| {
                Node::button(format!("Component {i}"), None)
                    .key(format!("nav-{i}"))
                    .class("nav", &rules)
                    .styled(Style::new().height(Size::Px(24.0)))
            })
            .collect(),
    )
    .key("sidebar")
    .styled(Style::new().width(Size::Px(210.0)).gap(3.0));
    let body = Node::column(
        (0..30)
            .map(|i| {
                Node::text(format!(
                    "Page {page}: component usage and native behavior, row {i}"
                ))
                .styled(Style::new().font_size(13.0))
            })
            .collect(),
    )
    .key("body")
    .styled(Style::new().width(Size::Px(650.0)).gap(4.0));
    Node::row(vec![sidebar, body]).styled(
        Style::new()
            .width(Size::Px(900.0))
            .height(Size::Px(700.0))
            .gap(20.0)
            .background(Color(255, 255, 255)),
    )
}
/// Build once so switches exercise the production reconciliation and native rewrite passes.
fn dashboard() -> (TestHarness<Element>, Routes) {
    let mut routes = Routes::new();
    let tree = Element::build(&scene(0), &mut routes);
    (
        TestHarness::create_with_size(theme::defaults(), tree, NativeSize::new(900.0, 700.0)),
        routes,
    )
}
/// Sidebar hover is paint-only and should not recompute the application's shared geometry.
#[test]
fn sidebar_hover_keeps_shared_geometry_clean() {
    let (mut harness, _) = dashboard();
    let baseline = harness.root_widget().geometry.borrow().computations;
    harness.mouse_move((100.0, 10.0));
    harness.mouse_move((100.0, 40.0));
    harness.mouse_move((100.0, 70.0));
    assert_eq!(
        harness.root_widget().geometry.borrow().computations,
        baseline
    );
}
/// Each published page needs one geometry computation, while persistent navigation IDs survive.
#[test]
fn switching_pages_computes_geometry_once_and_retains_sidebar() {
    let (mut harness, mut routes) = dashboard();
    let before = harness.root_widget().children[0].id();
    let baseline = harness.root_widget().geometry.borrow().computations;
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &scene(1), &mut routes));
    assert_eq!(harness.root_widget().children[0].id(), before);
    assert_eq!(
        harness.root_widget().geometry.borrow().computations,
        baseline + 1
    );
}
/// Opt-in native CPU timing: includes reconciliation/native layout, excludes GPU presentation and the worker.
#[test]
#[ignore = "development timing specimen; run with --ignored --nocapture"]
fn dashboard_navigation_timing() {
    let (mut harness, mut routes) = dashboard();
    let initial = harness.root_widget().geometry.borrow().computations;
    let mut times = Vec::new();
    for page in 1..=12 {
        let next = scene(page);
        let start = Instant::now();
        harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &next, &mut routes));
        times.push(start.elapsed().as_micros());
    }
    times.sort_unstable();
    println!(
        "navigation median_us={} max_us={} shared_layouts={}",
        times[times.len() / 2],
        times.last().unwrap(),
        harness.root_widget().geometry.borrow().computations - initial
    );
}

/// Window constraints and font publications must invalidate geometry; native focus-only passes may reuse it.
#[test]
fn geometry_cache_invalidates_for_resize_and_font_changes() {
    use masonry::{core::WindowEvent, dpi::PhysicalSize};
    let (mut harness, mut routes) = dashboard();
    let initial = harness.root_widget().geometry.borrow().computations;
    harness.process_window_event(WindowEvent::Resize(PhysicalSize::new(1000, 700)));
    assert_eq!(
        harness.root_widget().geometry.borrow().computations,
        initial + 1
    );
    let changed = scene(0).styled(Style::new().font_size(18.0));
    harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &changed, &mut routes));
    assert_eq!(
        harness.root_widget().geometry.borrow().computations,
        initial + 2
    );
}

/// Switching visible navigation buttons must not reveal a stale focused row or jump the sidebar.
#[test]
fn switching_visible_sidebar_items_does_not_scroll_to_stale_focus() {
    let page = |n| {
        let mut root = scene(n);
        let sidebar = std::mem::replace(&mut root.children[0], Node::empty());
        root.children[0] = Node::scroll_area(vec![sidebar])
            .styled(Style::new().width(Size::Px(210.0)).height(Size::Px(200.0)));
        root
    };
    let mut routes = Routes::new();
    let tree = Element::build(&page(0), &mut routes);
    let mut harness =
        TestHarness::create_with_size(theme::defaults(), tree, NativeSize::new(900.0, 700.0));
    for n in 0..12 {
        let wrapper = harness.root_widget().children[0].id();
        let nav = harness
            .get_widget_with_id(wrapper)
            .downcast::<Element>()
            .unwrap();
        let items = harness
            .get_widget_with_id(nav.children[0].id())
            .downcast::<Element>()
            .unwrap();
        let button = items.children[n % 3].id();
        let target = *routes
            .iter()
            .find(|(_, owner)| **owner == button)
            .unwrap()
            .0;
        harness.mouse_click_on(target);
        harness.edit_root_widget(|mut root| Element::reconcile(&mut root, &page(n), &mut routes));
        let root = harness.root_widget();
        let sidebar = harness
            .get_widget_with_id(root.children[0].id())
            .downcast::<Element>()
            .unwrap();
        assert_eq!(
            sidebar.scroll.as_ref().unwrap().borrow().metrics().offset.y,
            0.0,
            "switch {n}"
        );
    }
}
