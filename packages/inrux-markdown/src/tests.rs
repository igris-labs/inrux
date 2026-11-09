//! The optional document renderer depends only on shared Nodes, not native engines.
use super::*;
use inrux_components::{Node, NodeKind};
/// Collect readable text without relying on the renderer's structural wrapper depth.
fn texts(node: &Node, output: &mut Vec<String>) {
    if let NodeKind::Text(text) = &node.kind {
        output.push(text.clone());
    }
    for child in &node.children {
        texts(child, output);
    }
}
/// CommonMark supplies nesting and escaping; no HTML or image is executed/fetched by rendering.
#[test]
fn renders_headings_lists_quotes_code_and_literal_html_as_shared_nodes() {
    let doc = Markdown::parse(
        "# Heading\n\nA **strong** and *emphasized* paragraph.\n\n3. First\n4. Second\n\n> Quote\n\n```rust\nlet text = \"<Button />\";\n  indented\n```\n\n<div>literal</div>\n",
    );
    let view = doc.view();
    let mut output = vec![];
    texts(&view, &mut output);
    for expected in [
        "Heading",
        "A strong and emphasized paragraph.",
        "3.",
        "4.",
        "First",
        "Second",
        "Quote",
        "let text = \"<Button />\";\n  indented",
        "<div>literal</div>\n",
    ] {
        assert!(
            output.iter().any(|text| text == expected),
            "missing {expected:?}: {output:?}"
        );
    }
    assert_eq!(view.component_type, "Markdown");
}
/// Clones reuse the parse tree and repeated view construction does not parse or mutate it.
#[test]
fn empty_and_reused_documents_are_stable() {
    let doc = Markdown::parse("```iru\n<Input value=\"long code\" />\n```");
    assert!(Arc::ptr_eq(&doc.0, &doc.clone().0));
    assert!(Markdown::parse("").view().children.is_empty());
    let node = doc.view();
    let code = &node.children[0].children[0];
    assert_eq!(code.style().white_space, Some(inrux_style::WhiteSpace::Pre));
    assert_eq!(code.classes, vec!["md-code-text"]);
}
