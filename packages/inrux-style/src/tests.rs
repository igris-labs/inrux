//! Styling parity and validation regressions.

use super::*;
#[test]
fn css_and_rust_produce_identical_styles() {
    let rules = parse_css(".panel { padding: 24px; gap: 16px; background: #ffffff; width: 100%; }")
        .unwrap();
    assert_eq!(
        rules[0].style,
        Style::new()
            .padding(24.0)
            .gap(16.0)
            .background(Color(255, 255, 255))
            .width(Size::Percent(100.0))
    );
}
#[test]
fn rejects_unsupported_selectors_properties_and_bad_lengths() {
    for css in [
        ".a .b { gap: 1px; }",
        ".a { backdrop-filter: blur(1px); }",
        ".a { gap: NaNpx; }",
        ".a { width: -2%; }",
        ".a:visited {}",
        ".a { padding: 2em; }",
        ".a:hover { padding: 2px; }",
    ] {
        assert!(parse_css(css).is_err(), "accepted {css}");
    }
}
#[test]
fn overrides_preserve_unset_fields() {
    let mut style = Style::new().padding(8.0).gap(16.0);
    style.overlay(&Style::new().padding(24.0));
    assert_eq!(style, Style::new().padding(24.0).gap(16.0));
}

/// New layout properties share one representation across both styling frontends.
#[test]
fn constraints_alignment_wrap_and_comments_have_css_rust_parity() {
    use crate::{Align, Justify};
    let rules = parse_css("/* theme */ .row { min-width: 100px; max-width: 80%; min-height: 0; max-height: 500px; align-items: center; justify-content: space-between; flex-wrap: wrap; }").unwrap();
    assert_eq!(
        rules[0].style,
        Style::new()
            .min_width(Size::Px(100.0))
            .max_width(Size::Percent(80.0))
            .min_height(Size::Px(0.0))
            .max_height(Size::Px(500.0))
            .align_items(Align::Center)
            .justify_content(Justify::SpaceBetween)
            .wrap(true)
    );
    assert!(parse_css(".row:hover { flex-wrap: wrap; }").is_err());
    assert!(parse_css("/* unclosed").is_err());
}

/// CSS and Rust share named family values and preserve unrelated inherited typography.
#[test]
fn named_font_family_matches_rust_and_overlays() {
    let rules = parse_css(".text { font-family: \"Noto Sans\"; font-size: 15px; }").unwrap();
    let mut expected = Style::new().font_family("Noto Sans").font_size(15.0);
    assert_eq!(rules[0].style, expected);
    expected.overlay(&Style::new().font_family("Other"));
    assert_eq!(expected.font_family.as_deref(), Some("Other"));
    assert_eq!(expected.font_size, Some(15.0));
    assert!(parse_css(".text { font-family: Arial, serif; }").is_err());
}

/// Layout defaults use the same overrides through CSS and Rust.
#[test]
fn growth_and_automatic_margins_have_validated_css_rust_parity() {
    let rules = parse_css(".space { flex-grow: 2; margin-inline: auto; }").unwrap();
    assert_eq!(
        rules[0].style,
        Style::new().flex_grow(2.0).margin_inline_auto(true)
    );
    let reset = parse_css(".space { flex-grow: 0; margin-inline: 0; }").unwrap();
    let mut style = rules[0].style.clone();
    style.overlay(&reset[0].style);
    assert_eq!(style, Style::new().flex_grow(0.0).margin_inline_auto(false));
    for css in [
        ".x { flex-grow: -1; }",
        ".x { flex-grow: NaN; }",
        ".x { flex-grow: inf; }",
        ".x { flex-grow: 1px; }",
        ".x { margin-inline: 12px; }",
        ".x:hover { flex-grow: 1; }",
        ".x:focus-visible { margin-inline: auto; }",
    ] {
        assert!(parse_css(css).is_err(), "accepted {css}");
    }
}

/// Both authoring paths share the same bounded grid, direction and ratio representation.
#[test]
fn second_layout_group_has_css_and_rust_parity() {
    let css=parse_css(".layout { display: grid; grid-template-columns: repeat(3, 1fr); flex-direction: column; aspect-ratio: 16 / 9; }").unwrap();
    assert_eq!(
        css[0].style,
        Style::new()
            .display(Display::Grid)
            .grid_columns(3)
            .flex_direction(FlexDirection::Column)
            .aspect_ratio(16.0 / 9.0)
    );
    let mut style = css[0].style.clone();
    style.overlay(
        &Style::new()
            .display(Display::Flex)
            .grid_columns(1)
            .flex_direction(FlexDirection::Row)
            .aspect_ratio(1.0),
    );
    assert_eq!(
        style,
        Style::new()
            .display(Display::Flex)
            .grid_columns(1)
            .flex_direction(FlexDirection::Row)
            .aspect_ratio(1.0)
    );
}

/// Unsupported grid grammars and invalid ratios are errors rather than approximations.
#[test]
fn second_layout_group_rejects_invalid_css_and_interaction_geometry() {
    for declaration in [
        "display: block",
        "flex-direction: row-reverse",
        "grid-template-columns: repeat(0, 1fr)",
        "grid-template-columns: repeat(257, 1fr)",
        "grid-template-columns: repeat(2, 2fr)",
        "grid-template-columns: 1fr 1fr",
        "grid-template-columns: repeat(2, 1fr, 1fr)",
        "aspect-ratio: 0",
        "aspect-ratio: 16 / 0",
        "aspect-ratio: -1 / -2",
        "aspect-ratio: NaN",
        "aspect-ratio: inf",
        "aspect-ratio: 1e38 / 1e-38",
    ] {
        assert!(
            parse_css(&format!(".x {{ {declaration}; }}")).is_err(),
            "accepted {declaration}"
        );
    }
    for declaration in [
        "display: grid",
        "flex-direction: row",
        "grid-template-columns: repeat(2, 1fr)",
        "aspect-ratio: 1",
    ] {
        assert!(parse_css(&format!(".x:hover {{ {declaration}; }}")).is_err());
    }
}

/// Overflow has identical CSS/Rust semantics and cannot change geometry on hover.
#[test]
fn overflow_css_matches_rust_and_rejects_unsupported_values() {
    let rule = parse_css(".pane { overflow: auto; overflow-x: hidden; }").unwrap();
    assert_eq!(
        rule[0].style,
        Style::new()
            .overflow(Overflow::Auto)
            .overflow_x(Overflow::Hidden)
    );
    for css in [
        ".pane { overflow: clip; }",
        ".pane { overflow: auto hidden; }",
        ".pane:hover { overflow-y: auto; }",
    ] {
        assert!(parse_css(css).is_err(), "accepted {css}");
    }
}

/// Literal CSS colors compile to compact straight-alpha bytes without losing opaque constructor compatibility.
#[test]
fn css_color_literals_and_legacy_rgb_construction_share_one_value_type() {
    use crate::{Color, Rgba};
    for (input, expected) in [
        ("#123", Color(17, 34, 51)),
        ("#1234", Color::rgba(17, 34, 51, 68)),
        ("#11223380", Color::rgba(17, 34, 51, 128)),
        ("transparent", Color::rgba(0, 0, 0, 0)),
        ("rgb(255 0 0 / 50%)", Color::rgba(255, 0, 0, 128)),
        ("hsl(120 100% 50%)", Color(0, 255, 0)),
        ("oklch(100% 0 0)", Color(255, 255, 255)),
        ("red", Color(255, 0, 0)),
    ] {
        assert_eq!(Color::parse(input).unwrap(), expected, "{input}");
    }
    let Rgba(r, g, b, a) = Color(1, 2, 3);
    assert_eq!((r, g, b, a), (1, 2, 3, 255));
    assert_eq!(std::mem::size_of::<Color>(), 4);
    assert!(Color::hex("#123").is_err());
    for bad in [
        "currentColor",
        "color-mix(in srgb, red, blue)",
        "oklch(from red l c h)",
        "#12345",
        "red trailing",
    ] {
        assert!(Color::parse(bad).is_err(), "{bad}");
    }
    let rules=crate::parse_css(":root {--ink:oklch(24.35% 0 0);--fill:rgb(12 34 56 / 50%);} .x {background:var(--fill);border-color:transparent;color:var(--ink);}").unwrap();
    assert_eq!(
        rules[0].style.background,
        Some(Color::rgba(12, 34, 56, 128))
    );
    assert_eq!(rules[0].style.border_color, Some(Color::rgba(0, 0, 0, 0)));
}

/// Functional scrollbar colors retain internal whitespace, alpha and the existing two-value contract.
#[test]
fn functional_scrollbar_colors_preserve_alpha() {
    let rules = crate::parse_css(".x {scrollbar-color: rgb(20 40 60 / 50%) transparent;}").unwrap();
    assert_eq!(
        rules[0].style.scrollbar_colors,
        Some((
            crate::Color::rgba(20, 40, 60, 128),
            crate::Color::rgba(0, 0, 0, 0)
        ))
    );
    assert!(crate::parse_css(".x {scrollbar-color: red green blue;}").is_err());
}
