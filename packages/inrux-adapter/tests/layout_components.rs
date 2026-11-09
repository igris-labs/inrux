//! Physical geometry contracts shared by all current layout bindings.
#![cfg(feature = "taffy-layout")]
use inrux_adapter::layout::{self, LayoutBox};
use inrux_components::Node;
use inrux_style::{Size, Style, parse_css};

/// Keep these layout tests independent of native font shaping.
fn fixed(width: f32, height: f32) -> Node {
    Node::box_container(vec![]).styled(Style::new().width(Size::Px(width)).height(Size::Px(height)))
}
/// Execute the same validated fallback selected by all three adapters.
fn geometry(node: &Node, width: f32) -> LayoutBox {
    layout::compute(node, width, None, |_| (0.0, 0.0)).unwrap()
}

#[test]
fn container_centers_at_wide_width_and_fills_narrow_parent() {
    let node = Node::box_container(vec![Node::container(vec![fixed(20.0, 30.0)])]);
    let wide = geometry(&node, 1600.0);
    assert_eq!(
        (wide.children[0].x, wide.children[0].width),
        (200.0, 1200.0)
    );
    let narrow = geometry(&node, 320.0);
    assert_eq!(
        (narrow.children[0].x, narrow.children[0].width),
        (0.0, 320.0)
    );
}

#[test]
fn padded_container_includes_padding_inside_its_maximum_width() {
    let node = Node::container(vec![fixed(20.0, 30.0)]).styled(Style::new().padding(24.0));
    let result = geometry(&node, 1600.0);
    assert_eq!((result.width, result.height), (1200.0, 78.0));
    assert_eq!((result.children[0].x, result.children[0].y), (24.0, 24.0));
}

#[test]
fn center_uses_available_space_on_both_axes() {
    let node = Node::center(vec![fixed(40.0, 20.0)]).styled(Style::new().height(Size::Px(100.0)));
    let result = geometry(&node, 200.0);
    assert_eq!((result.children[0].x, result.children[0].y), (80.0, 40.0));
}

#[test]
fn center_does_not_override_explicit_child_fill() {
    let node = Node::center(vec![Node::box_container(vec![fixed(20.0, 10.0)])]);
    let result = geometry(&node, 200.0);
    assert_eq!(
        (result.children[0].x, result.children[0].width),
        (0.0, 200.0)
    );
}

#[test]
fn horizontal_spacers_share_free_space_and_preserve_sibling_order() {
    let node = Node::row(vec![
        fixed(40.0, 20.0),
        Node::spacer(),
        Node::spacer(),
        fixed(40.0, 20.0),
    ]);
    let result = geometry(&node, 200.0);
    assert_eq!(result.children[1].width, 60.0);
    assert_eq!(result.children[2].width, 60.0);
    assert_eq!(result.children[3].x, 160.0);
}

#[test]
fn vertical_spacer_requires_bounded_main_axis() {
    let contents = || vec![fixed(20.0, 20.0), Node::spacer(), fixed(20.0, 20.0)];
    let bounded = Node::column(contents()).styled(Style::new().height(Size::Px(100.0)));
    let result = geometry(&bounded, 200.0);
    assert_eq!(
        (result.children[1].height, result.children[2].y),
        (60.0, 80.0)
    );
    let unbounded = geometry(&Node::column(contents()), 200.0);
    assert_eq!(unbounded.children[1].height, 0.0);
}

#[test]
fn spacer_does_not_create_space_when_parent_is_full() {
    let node = Node::row(vec![fixed(40.0, 20.0), Node::spacer(), fixed(40.0, 20.0)]);
    let result = geometry(&node, 80.0);
    assert_eq!(
        (result.children[1].width, result.children[2].x),
        (0.0, 40.0)
    );
}

#[test]
fn css_and_rust_override_component_defaults_with_identical_geometry() {
    let css = parse_css(".narrow { max-width: 240px; margin-inline: 0; }").unwrap();
    let rust = Node::container(vec![fixed(20.0, 20.0)]).styled(
        Style::new()
            .max_width(Size::Px(240.0))
            .margin_inline_auto(false),
    );
    let mut styled = Node::container(vec![fixed(20.0, 20.0)]).class("narrow", &[]);
    styled.apply_globals(&css);
    assert_eq!(geometry(&styled, 800.0), geometry(&rust, 800.0));
    assert_eq!(geometry(&styled, 800.0).width, 240.0);
}

#[test]
fn invalid_rust_growth_is_rejected_before_layout() {
    for value in [-1.0, f32::NAN, f32::INFINITY] {
        let node = Node::spacer().styled(Style::new().flex_grow(value));
        assert!(layout::validate(&node).is_err());
    }
}

/// Direction changes rearrange geometry without rebuilding or re-keying descriptions.
#[test]
fn flex_direction_changes_axis_and_preserves_child_identity() {
    use inrux_style::FlexDirection;
    let node = Node::flex(vec![
        fixed(40.0, 20.0).key("first"),
        fixed(40.0, 20.0).key("second"),
    ])
    .styled(Style::new().gap(10.0));
    let horizontal = geometry(&node, 200.0);
    assert_eq!(
        (horizontal.children[1].x, horizontal.children[1].y),
        (50.0, 0.0)
    );
    let column = node
        .clone()
        .styled(Style::new().flex_direction(FlexDirection::Column));
    let vertical = geometry(&column, 200.0);
    assert_eq!(
        (vertical.children[1].x, vertical.children[1].y),
        (0.0, 30.0)
    );
    assert_eq!(column.children[1].key, node.children[1].key);
}

/// Uniform tracks include padding/gaps and wrap source-ordered children into automatic rows.
#[test]
fn grid_sizes_equal_columns_and_places_rows_in_source_order() {
    let children = (0..5)
        .map(|_| Node::box_container(vec![]).styled(Style::new().height(Size::Px(20.0))))
        .collect();
    let node = Node::grid(children).styled(Style::new().grid_columns(3).padding(10.0).gap(10.0));
    let boxes = geometry(&node, 340.0);
    assert_eq!(boxes.height, 70.0);
    assert!(boxes.children.iter().all(|child| child.width == 100.0));
    assert_eq!((boxes.children[0].x, boxes.children[0].y), (10.0, 10.0));
    assert_eq!((boxes.children[2].x, boxes.children[2].y), (230.0, 10.0));
    assert_eq!((boxes.children[3].x, boxes.children[3].y), (10.0, 40.0));
}

/// Narrow tracks remain equal even when intrinsic content is much wider than the cell.
#[test]
fn grid_narrow_tracks_do_not_expand_to_intrinsic_text_width() {
    let node = Node::grid(vec![
        Node::text("very long".into()),
        Node::text("short".into()),
    ]);
    let boxes = layout::compute(&node, 80.0, None, |request| {
        (request.text.len() as f32 * 100.0, 20.0)
    })
    .unwrap();
    assert_eq!(boxes.width, 80.0);
    assert_eq!(
        (boxes.children[0].width, boxes.children[1].width),
        (40.0, 40.0)
    );
    assert_eq!(boxes.children[1].x, 40.0);
}

/// Empty grids reserve no phantom row height, and source styles can replace their defaults.
#[test]
fn empty_grid_and_css_overrides_keep_valid_geometry() {
    let empty = geometry(&Node::grid(vec![]), 200.0);
    assert_eq!((empty.width, empty.height), (200.0, 0.0));
    let rules = parse_css(".one { grid-template-columns: repeat(1, 1fr); gap: 8px; }").unwrap();
    let children = || vec![fixed(40.0, 20.0), fixed(40.0, 20.0)];
    let css = Node::grid(children()).class("one", &rules);
    let rust = Node::grid(children()).styled(Style::new().grid_columns(1).gap(8.0));
    assert_eq!(geometry(&css, 200.0), geometry(&rust, 200.0));
    assert_eq!(geometry(&css, 200.0).children[1].y, 28.0);
}

/// An automatic axis follows ratio while explicit dimensions retain their authored size.
#[test]
fn aspect_ratio_supports_width_height_and_explicit_size_cases() {
    let width =
        Node::aspect_ratio(vec![]).styled(Style::new().width(Size::Px(200.0)).aspect_ratio(2.0));
    let boxes = geometry(&width, 300.0);
    assert_eq!((boxes.width, boxes.height), (200.0, 100.0));
    let height = Node::aspect_ratio(vec![]).styled(
        Style::new()
            .width(Size::Content)
            .height(Size::Px(80.0))
            .aspect_ratio(2.0),
    );
    let boxes = geometry(&height, 300.0);
    assert_eq!((boxes.width, boxes.height), (160.0, 80.0));
    let explicit = width.styled(Style::new().height(Size::Px(40.0)));
    assert_eq!(geometry(&explicit, 300.0).height, 40.0);
    for node in [
        Node::aspect_ratio(vec![fixed(10.0, 10.0)]),
        Node::grid(vec![]).styled(Style::new().aspect_ratio(1.0)),
    ] {
        let explicit = node
            .clone()
            .styled(Style::new().width(Size::Px(200.0)).height(Size::Px(40.0)));
        assert_eq!(geometry(&explicit, 300.0).height, 40.0);
        let bounded = node.styled(Style::new().max_height(Size::Px(60.0)));
        assert_eq!(geometry(&bounded, 200.0).height, 60.0);
    }
}

/// Bounds remain authoritative even when they prevent the preferred ratio.
#[test]
fn aspect_ratio_respects_constraints_and_parent_resize() {
    let square = Node::aspect_ratio(vec![]);
    assert_eq!(geometry(&square, 120.0).height, 120.0);
    assert_eq!(geometry(&square, 240.0).height, 240.0);
    let constrained = square.styled(Style::new().max_height(Size::Px(60.0)));
    assert_eq!(geometry(&constrained, 200.0).height, 60.0);
}

/// Rust callers receive the same pre-layout validation as CSS authors.
#[test]
fn invalid_columns_ratios_and_grid_leaves_fail_before_provider_execution() {
    use inrux_style::Display;
    for count in [0, 257, u16::MAX] {
        assert!(
            layout::validate(&Node::grid(vec![]).styled(Style::new().grid_columns(count))).is_err()
        );
    }
    for ratio in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(
            layout::validate(&Node::aspect_ratio(vec![]).styled(Style::new().aspect_ratio(ratio)))
                .is_err()
        );
    }
    assert!(
        layout::validate(&Node::text("leaf".into()).styled(Style::new().display(Display::Grid)))
            .is_err()
    );
    assert!(
        layout::validate(&Node::text("leaf".into()).styled(Style::new().aspect_ratio(1.0)))
            .is_err()
    );
}

/// Intrinsic Center sizing can exhaust the content width before subtracting child padding.
#[test]
fn center_measures_padded_intrinsic_text_without_negative_provider_widths() {
    let node = Node::center(vec![
        Node::text("Centered on both axes".into()).styled(Style::new().padding(10.0)),
    ])
    .styled(Style::new().height(Size::Px(150.0)).padding(12.0));
    for available in [0.0, 10.0, 400.0] {
        let mut measured_zero_width = false;
        let result = layout::compute(&node, available, None, |request| {
            assert!(request.max_width >= 0.0);
            measured_zero_width |= request.max_width == 0.0;
            (request.max_width.min(120.0), 18.0)
        })
        .unwrap();
        if available == 0.0 {
            assert!(
                measured_zero_width,
                "exercise the padding-exhausted intrinsic probe"
            );
        }
        if available == 400.0 {
            let child = &result.children[0];
            assert_eq!(
                (child.x, child.y, child.width, child.height),
                (130.0, 56.0, 140.0, 38.0)
            );
        }
    }
}

/// Absolute rails do not consume flow space and follow parent insets on resize.
#[test]
fn absolute_overlay_uses_parent_bounds_without_shrinking_the_scroller() {
    use inrux_style::Position;
    let root = Node::column(vec![
        fixed(600.0, 400.0),
        fixed(140.0, 100.0).styled(
            Style::new()
                .position(Position::Absolute)
                .top(Size::Px(24.0))
                .right(Size::Px(20.0)),
        ),
    ])
    .styled(Style::new().width(Size::Fill).height(Size::Px(400.0)));
    let layout = geometry(&root, 600.0);
    assert_eq!(layout.children[0].width, 600.0);
    assert_eq!((layout.children[1].x, layout.children[1].y), (440.0, 24.0));
}

/// Native preformatted text receives an unbounded shaping width and overflows into a horizontal viewport.
#[test]
fn preformatted_text_keeps_long_lines_instead_of_wrapping_to_viewport_width() {
    use inrux_style::{Overflow, WhiteSpace};
    let node = Node::scroll_area(vec![
        Node::text("x".repeat(100)).styled(
            Style::new()
                .white_space(WhiteSpace::Pre)
                .width(Size::Content),
        ),
    ])
    .styled(
        Style::new()
            .width(Size::Px(200.0))
            .height(Size::Content)
            .overflow_x(Overflow::Auto)
            .overflow_y(Overflow::Hidden),
    );
    let geometry = layout::compute(&node, 200.0, None, |request| {
        assert!(request.max_width.is_infinite());
        (800.0, 20.0)
    })
    .unwrap();
    assert_eq!(geometry.width, 200.0);
    // The text box may stretch to the viewport; its unwrapped glyph extent must still overflow.
    assert!(geometry.children[0].content_width >= 800.0);
    assert!(geometry.content_width >= 800.0);
}
