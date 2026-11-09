//! Content compositions use the same layout and styling on every current binding.
#![cfg(feature = "taffy-layout")]
use inrux_adapter::layout;
use inrux_components::{Node, NodeKind};
use inrux_style::{Color, Size, Style, parse_css};

/// Deterministic metrics isolate component geometry from native font rasterization.
fn geometry(node: &Node, width: f32) -> layout::LayoutBox {
    layout::compute(node, width, None, |request| {
        (request.text.chars().count() as f32 * 6.0, 16.0)
    })
    .unwrap()
}

#[test]
fn card_reserves_padding_and_gap_for_ordered_children() {
    let node = Node::card(vec![
        Node::text("First".into()).key("first"),
        Node::text("Second".into()).key("second"),
    ]);
    let bounds = geometry(&node, 300.0);
    assert_eq!((bounds.width, bounds.height), (300.0, 76.0));
    assert_eq!((bounds.children[0].x, bounds.children[0].y), (16.0, 16.0));
    assert_eq!((bounds.children[1].x, bounds.children[1].y), (16.0, 44.0));
    assert_eq!(bounds.children[0].width, 268.0);
    assert_eq!(node.children[1].key.as_deref(), Some("second"));
    assert_eq!(geometry(&Node::card(vec![]), 300.0).height, 32.0);
}

#[test]
fn separators_have_bounded_geometry_and_can_be_explicitly_sized() {
    let horizontal = geometry(&Node::separator(false), 240.0);
    assert_eq!((horizontal.width, horizontal.height), (240.0, 1.0));
    let vertical = geometry(&Node::separator(true), 240.0);
    assert_eq!((vertical.width, vertical.height), (1.0, 24.0));
    let sized = Node::separator(true).styled(Style::new().height(Size::Px(60.0)));
    let row = geometry(&Node::row(vec![sized]), 240.0);
    assert_eq!((row.children[0].width, row.children[0].height), (1.0, 60.0));
    assert_eq!(geometry(&Node::separator(false), 80.0).width, 80.0);
}

#[test]
fn badge_uses_text_metrics_padding_and_normal_flex_stretch() {
    let badge = || Node::badge("Saved".into());
    let compact = geometry(&Node::row(vec![badge()]), 240.0);
    assert_eq!(
        (compact.children[0].width, compact.children[0].height),
        (38.0, 24.0)
    );
    let stretched = geometry(&Node::column(vec![badge()]), 240.0);
    assert_eq!(stretched.children[0].width, 240.0);
    let empty = geometry(&Node::row(vec![Node::badge(String::new())]), 240.0);
    assert!(empty.children[0].width.is_finite());
    assert!(empty.children[0].height >= 8.0);
    let long = Node::badge("Unusually long status message".into())
        .styled(Style::new().width(Size::Px(80.0)));
    let bounds = geometry(&long, 240.0);
    assert_eq!(bounds.width, 80.0);
    assert!(matches!(long.kind, NodeKind::Text(_)));
}

#[test]
fn css_and_rust_customize_content_defaults_identically() {
    let css = parse_css(".custom { padding: 8px; gap: 4px; background: #123456; border-radius: 2px; border-width: 0; }").unwrap();
    let style = Style::new()
        .padding(8.0)
        .gap(4.0)
        .background(Color(18, 52, 86))
        .rounded(2.0)
        .border_width(0.0);
    for node in [
        Node::card(vec![Node::badge("Status".into())]),
        Node::badge("Status".into()),
        Node::separator(false),
    ] {
        let from_css = node.clone().class("custom", &css);
        let from_rust = node.styled(style.clone());
        assert_eq!(from_css.style(), from_rust.style());
        assert_eq!(geometry(&from_css, 240.0), geometry(&from_rust, 240.0));
    }
}

#[test]
fn global_scoped_and_explicit_layers_override_defaults_and_preserve_inheritance() {
    let global = parse_css(".custom { padding: 6px; color: #123456; font-size: 18px; }").unwrap();
    let scoped = parse_css(".custom { padding: 8px; }").unwrap();
    let mut card = Node::card(vec![
        Node::text("Inherited".into()),
        Node::badge("Label".into()),
    ])
    .class("custom", &scoped)
    .styled(Style::new().padding(10.0));
    card.apply_globals(&global);
    card.inherit(&Style::new().font_family("Noto Sans"));
    assert_eq!(card.style().padding, Some(10.0));
    assert_eq!(card.children[0].style().color, Some(Color(18, 52, 86)));
    assert_eq!(card.children[0].style().font_size, Some(18.0));
    assert_eq!(card.children[1].style().font_size, Some(12.0));
    assert_eq!(
        card.children[1].style().font_family.as_deref(),
        Some("Noto Sans")
    );
}
