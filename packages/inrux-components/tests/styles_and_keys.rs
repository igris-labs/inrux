//! Shared render descriptors preserve styling boundaries and stable sibling identity.
use inrux_components::Node;
use inrux_style::{Color, Style, parse_css};
/// Scoped classes win over global classes; Rust builders remain the explicit final layer.
#[test]
fn scoped_global_and_inline_layers_have_deterministic_precedence() {
    let local = parse_css(".panel { color: #112233; }").unwrap();
    let global = parse_css(".panel { color: #ffffff; padding: 12px; }").unwrap();
    let mut tree = Node::column(vec![
        Node::text("local".into()).class("panel", &local),
        Node::text("global".into()).class("panel", &[]),
    ]);
    tree.apply_globals(&global);
    assert_eq!(tree.children[0].style().color, Some(Color(17, 34, 51)));
    assert_eq!(tree.children[1].style().color, Some(Color(255, 255, 255)));
    assert_eq!(tree.children[0].style().padding, Some(12.0));
    assert_eq!(
        tree.children[0]
            .clone()
            .styled(Style::new().color(Color(1, 2, 3)))
            .style()
            .color,
        Some(Color(1, 2, 3))
    );
}
/// Keys are local to the parent and duplicate siblings fail before painting.
#[test]
fn duplicate_keys_are_rejected_but_nested_names_can_repeat() {
    let duplicate = Node::column(vec![
        Node::text("a".into()).key("same"),
        Node::text("b".into()).key("same"),
    ]);
    assert!(duplicate.validate_keys().unwrap_err().contains("same"));
    let valid = Node::column(vec![
        Node::column(vec![Node::text("a".into()).key("same")]).key("first"),
        Node::column(vec![Node::text("b".into()).key("same")]).key("second"),
    ]);
    assert!(valid.validate_keys().is_ok());
}

/// Library defaults stay below all application style layers and retain structural identity.
#[test]
fn component_defaults_yield_to_global_scoped_and_explicit_styles() {
    use inrux_style::Size;
    let global = parse_css(".panel { max-width: 900px; }").unwrap();
    let scoped = parse_css(".panel { max-width: 700px; }").unwrap();
    let make = || Node::container(vec![Node::text("child".into()).key("child")]).key("panel");
    let mut global_only = make().class("panel", &[]);
    global_only.apply_globals(&global);
    assert_eq!(global_only.style().max_width, Some(Size::Px(900.0)));
    let mut local = make().class("panel", &scoped);
    local.apply_globals(&global);
    assert_eq!(local.style().max_width, Some(Size::Px(700.0)));
    let explicit = local.styled(Style::new().max_width(Size::Px(500.0)));
    assert_eq!(explicit.style().max_width, Some(Size::Px(500.0)));
    assert_eq!(explicit.key.as_deref(), Some("panel"));
    assert_eq!(explicit.children[0].key.as_deref(), Some("child"));
}

/// ID selectors use style identity while keys continue to own native retention identity.
#[test]
fn indexed_specificity_media_and_inherited_scrollbars_resolve_together() {
    use inrux_style::{CompiledStyles, Viewport};
    let rules = parse_css("Button { color: #112233; } #save.action { color: #445566; } .action { color: #778899; } @media (max-width: 900px) { .action { padding: 8px; } } .root { scrollbar-color: #123456 #abcdef; }").unwrap();
    let sheet = CompiledStyles::new(rules);
    let mut tree = Node::column(vec![
        Node::button("Save".into(), None)
            .classes("action action")
            .id("save")
            .key("retained"),
    ])
    .classes("root");
    tree.resolve_styles(
        &[&sheet],
        Viewport {
            width: 900.0,
            height: 820.0,
        },
    );
    tree.inherit(&Style::new());
    let child = &tree.children[0];
    assert_eq!(child.style().color, Some(Color(68, 85, 102)));
    assert_eq!(child.style().padding, Some(8.0));
    assert_eq!(
        child.style().scrollbar_colors,
        Some((Color(18, 52, 86), Color(171, 205, 239)))
    );
    assert_eq!(child.key.as_deref(), Some("retained"));
    assert_eq!(child.classes, vec!["action"]);
    tree.resolve_styles(
        &[&sheet],
        Viewport {
            width: 1200.0,
            height: 860.0,
        },
    );
    assert_eq!(tree.children[0].style().padding, None);
}
