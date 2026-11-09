//! Public part/state cascade must not leak style into the owner or mutate component behavior.
use inrux_components::Node;
use inrux_style::{Color, CompiledStyles, Part, State, Style, Viewport, parse_css};
use std::sync::Arc;
/// Owners and public parts keep separate cascades with ordinary specificity and explicit overrides.
#[test]
fn parts_keep_owner_state_and_explicit_override_precedence() {
    let sheet = Arc::new(CompiledStyles::new(
        parse_css(
            r#"
        Switch { color:#111111; }
        Switch::part(track) { width:48px; height:28px; background:#aaaaaa; }
        .choice::part(track):checked { background:#112233; }
        Switch::part(track):hover { background:#224466; }
        Switch::part(label) { color:#445566; font-size:18px; }
    "#,
        )
        .unwrap(),
    ));
    let mut node = Node::switch("Sync".into(), true, inrux_core::Emit::new(|_| Ok(())))
        .classes("choice")
        .with_styles(sheet);
    node.resolve_styles(&[], Viewport::default());
    assert_eq!(node.style().background, None);
    assert_eq!(
        node.part_style(Part::Track, false, false).background,
        Some(Color(17, 34, 51))
    );
    assert_eq!(
        node.part_style(Part::Track, true, false).background,
        Some(Color(17, 34, 51)),
        "class specificity beats later type hover"
    );
    node = node.styled_part(Part::Track, Style::new().background(Color(9, 8, 7)));
    assert_eq!(
        node.part_style(Part::Track, true, true).background,
        Some(Color(9, 8, 7))
    );
    assert_eq!(
        node.part_style(Part::Label, false, false).font_size,
        Some(18.0)
    );
    assert!(node.matches_style_state(State::Checked, false, false));
    node.disabled = true;
    assert!(!node.matches_style_state(State::Hover, true, true));
    assert!(!node.matches_style_state(State::Focus, true, true));
    assert!(node.matches_style_state(State::Checked, true, true));
    let snapshot = node.paint_snapshot();
    assert!(snapshot.children.is_empty());
    assert_eq!(
        snapshot.part_style(Part::Track, false, false),
        node.part_style(Part::Track, false, false)
    );
}
