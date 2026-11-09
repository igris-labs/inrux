//! End-to-end stylesheet compilation, indexed matching and rejection boundaries.
use crate::*;
/// Specificity wins over source order and the order of classes on a node.
#[test]
fn compound_selectors_lists_and_source_order_have_a_css_cascade() {
    let sheet = CompiledStyles::new(parse_css("#save { color: #112233; } Button.primary { color: #445566; } .primary, .secondary { color: #778899; } Button { padding: 8px; } .primary { padding: 12px; }").unwrap());
    let classes = vec!["primary".into(), "extra".into()];
    let mut result = Style::new();
    for rule in sheet.matching("Button", Some("save"), &classes, Viewport::default()) {
        result.overlay(&rule.style);
    }
    assert_eq!(result.color, Some(Color(17, 34, 51)));
    assert_eq!(result.padding, Some(12.0));
    assert_eq!(
        sheet
            .matching("Input", None, &[], Viewport::default())
            .count(),
        0
    );
}
/// Constants resolve forward references, overrides and nested fallbacks before typed parsing.
#[test]
fn tokens_resolve_once_with_cycles_missing_values_and_quoted_delimiters_checked() {
    let rules = parse_css(":root { --foreground: var(--neutral); --neutral: #123456; --gap: 8px; } .x { color: var(--foreground); padding: var(--missing,var(--gap)); font-family: 'A; B: C'; } :root { --gap: 12px; }").unwrap();
    assert_eq!(
        rules[0].style,
        Style::new()
            .color(Color(18, 52, 86))
            .padding(12.0)
            .font_family("A; B: C")
    );
    for css in [
        ":root { --a: var(--b); --b: var(--a); }",
        ".x { color: var(--undefined); }",
        ".x { --gap: 1px; }",
        "@media (min-width: 1px) { :root { --x: 2px; } }",
        ".x { color: var(--x; }",
        ".x { color: #ffffff !important; }",
    ] {
        assert!(parse_css(css).is_err(), "accepted {css}");
    }
}
/// Nested conjunctions are inclusive, source-ordered and cannot escape their parent bounds.
#[test]
fn media_rules_match_width_and_height_boundaries() {
    let sheet = CompiledStyles::new(parse_css(".x { padding: 1px; } @media (min-width: 800px) and (max-height: 900px) { .x { padding: 2px; } @media (max-width: 1000px) { .x { padding: 3px; } } }").unwrap());
    for (width, height, expected) in [
        (799.0, 800.0, 1.0),
        (800.0, 900.0, 3.0),
        (1000.0, 900.0, 3.0),
        (1001.0, 900.0, 2.0),
        (900.0, 901.0, 1.0),
    ] {
        let mut style = Style::new();
        for rule in sheet.matching("Text", None, &["x".into()], Viewport { width, height }) {
            style.overlay(&rule.style);
        }
        assert_eq!(style.padding, Some(expected));
    }
    for css in [
        "@media (orientation: portrait) {}",
        "@media (min-width: NaNpx) {}",
        "@media screen {}",
        ".x > .y {}",
        ".x[disabled] {}",
        "*🦀 {}",
        " {}",
        ".x, {}",
    ] {
        assert!(parse_css(css).is_err(), "accepted {css}");
    }
}
/// The anchor index agrees with exhaustive matching even with repeated authored classes.
#[test]
fn index_matches_exhaustive_cascade() {
    let rules = parse_css("* { padding: 1px; } Text { padding: 2px; } .x { padding: 3px; } .x.y { padding: 4px; } #a.x { padding: 5px; } Text#b.y { padding: 6px; }").unwrap();
    let sheet = CompiledStyles::new(rules.clone());
    for kind in ["Text", "Button"] {
        for id in [None, Some("a"), Some("b")] {
            for names in [
                vec![],
                vec!["x".into()],
                vec!["x".into(), "y".into(), "x".into()],
            ] {
                let mut expected: Vec<_> = rules
                    .iter()
                    .enumerate()
                    .filter(|(_, rule)| rule.selector.matches(kind, id, &names))
                    .collect();
                expected.sort_by_key(|(i, r)| (r.selector.specificity(r.state), *i));
                assert_eq!(
                    sheet
                        .matching(kind, id, &names, Viewport::default())
                        .collect::<Vec<_>>(),
                    expected.into_iter().map(|(_, r)| r).collect::<Vec<_>>()
                );
            }
        }
    }
}

/// Scrollbar palettes preserve CSS/Rust parity and do not alter viewport geometry.
#[test]
fn scrollbar_colors_and_quoted_token_text_are_typed_and_validated() {
    let rule =
        parse_css(".x { scrollbar-color: #123456 #abcdef; font-family: 'Literal var(--name)'; }")
            .unwrap();
    assert_eq!(
        rule[0].style,
        Style::new()
            .scrollbar_colors(Color(18, 52, 86), Color(171, 205, 239))
            .font_family("Literal var(--name)")
    );
    assert_eq!(
        parse_css(".x {scrollbar-color:red blue;}").unwrap()[0]
            .style
            .scrollbar_colors,
        Some((Color(255, 0, 0), Color(0, 0, 255)))
    );
    for css in [
        ".x { scrollbar-color: unknown-color blue; }",
        ".x { scrollbar-color: #123456; }",
        ".x:hover { font-family: 'Other'; }",
        ".x:hover { scrollbar-color: #123456 #abcdef; }",
    ] {
        assert!(parse_css(css).is_err());
    }
}
/// Exponential constants and deeply nested fallback graphs return diagnostics instead of exhausting memory.
#[test]
fn token_expansion_has_bounded_work_and_output() {
    let mut css = String::from(":root { --v0: x; ");
    for i in 1..32 {
        css.push_str(&format!("--v{i}: var(--v{})var(--v{});", i - 1, i - 1));
    }
    css.push('}');
    assert!(parse_css(&css).is_err());
}

/// Shared positioning and preformatted declarations retain typed CSS/Rust parity.
#[test]
fn positioning_and_preformatted_text_are_geometry_not_interaction_paint() {
    let rules =
        parse_css(".rail { position: absolute; top: 24px; right: 5%; white-space: pre; }").unwrap();
    assert_eq!(
        rules[0].style,
        Style::new()
            .position(Position::Absolute)
            .top(Size::Px(24.0))
            .right(Size::Percent(5.0))
            .white_space(WhiteSpace::Pre)
    );
    for css in [
        ".x { position: fixed; }",
        ".x { top: -1px; }",
        ".x:hover { position: absolute; }",
        ".x:hover { white-space: pre; }",
    ] {
        assert!(parse_css(css).is_err());
    }
}

/// Gap shorthand and axis longhands follow declaration order and share Rust values.
#[test]
fn accent_and_gap_axes_compile_with_css_rust_parity_and_source_order() {
    let rules = parse_css(".x { gap: 12px 4px; row-gap: 20px; accent-color: #123456; }").unwrap();
    assert_eq!(
        rules[0].style,
        Style::new()
            .gap(12.0)
            .column_gap(4.0)
            .row_gap(20.0)
            .accent_color(Color(18, 52, 86))
    );
    let reset = parse_css(".x { row-gap: 99px; column-gap: 20px; gap: 8px; }").unwrap();
    assert_eq!(reset[0].style, Style::new().gap(8.0));
    let mut overlay = Style::new().gap(12.0).row_gap(99.0);
    overlay.overlay(&reset[0].style);
    assert_eq!(overlay, Style::new().gap(8.0));
    for css in [
        ".x { row-gap: -1px; }",
        ".x { gap: 1px 2px 3px; }",
        ".x:hover { column-gap: 4px; }",
        ".x:hover { accent-color: #123456; }",
    ] {
        assert!(parse_css(css).is_err());
    }
}

/// Parts are typed and indexed by the owner, with explicit syntax/state diagnostics.
#[test]
fn public_part_selectors_compile_without_runtime_parsing() {
    let rules=crate::parse_css("Switch.choice::part(track):checked {background:#112233;} Input::part(placeholder):disabled {color:#445566;}").unwrap();
    assert_eq!(rules[0].selector.part, Some(crate::Part::Track));
    assert_eq!(rules[0].state, crate::State::Checked);
    let sheet = crate::CompiledStyles::new(rules);
    assert_eq!(
        sheet
            .matching(
                "Switch",
                None,
                &["choice".into()],
                crate::Viewport::default()
            )
            .count(),
        1
    );
    for bad in [
        "Switch::part(native-widget) {color:#112233;}",
        "Switch::part(track):hover {width:32px;}",
        "Switch::part(track):checked:hover {color:#112233;}",
        "Switch::part(track {color:#112233;}",
    ] {
        assert!(crate::parse_css(bad).is_err(), "{bad}");
    }
}

/// Physical margins preserve shorthand/auto/longhand source order and reject unsupported units.
#[test]
fn physical_margin_cascade_and_rust_parity() {
    let rules =
        crate::parse_css(".x {margin:1px 2px 3px 4px;margin-inline:auto;margin-left:-5px;}")
            .unwrap();
    let expected = crate::Style::new()
        .margin_top(1.0)
        .margin_bottom(3.0)
        .margin_inline_auto(true)
        .margin_left(-5.0);
    assert_eq!(rules[0].style, expected);
    let mut base = crate::Style::new().margin(20.0);
    base.overlay(&expected);
    assert_eq!(base.margin_right, None);
    assert_eq!(base.margin_left, Some(-5.0));
    base.overlay(&crate::Style::new().margin(8.0));
    assert_eq!(base.margin_left, Some(8.0));
    assert_eq!(base.margin_inline_auto, Some(false));
    for bad in [
        ".x{margin:1px 2px 3px 4px 5px;}",
        ".x{margin:2px auto;}",
        ".x{margin-left:NaNpx;}",
        ".x:hover{margin:2px;}",
    ] {
        assert!(crate::parse_css(bad).is_err(), "{bad}");
    }
}
