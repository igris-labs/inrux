//! Built-in content syntax lowers to shared constructors with authored diagnostics.
use inrux_compiler::compile;

fn source(view: &str) -> String {
    format!("<view>\n{view}\n</view>\n<rust>\npub struct State;\n</rust>\n")
}

#[test]
fn content_tags_lower_dynamic_values_styles_and_decorative_orientation() {
    let output = compile(&source(r#"<Card class="surface" key="card"><Badge :text={state.status.clone()} /><Separator /><Separator :vertical={state.vertical} height={Size::Px(40.0)} /><Button text="Save" @click={save} /></Card>"#)).unwrap();
    syn::parse_file(&output).unwrap();
    let compact: String = output.chars().filter(|c| !c.is_whitespace()).collect();
    for expected in [
        "Node::card(",
        "Node::badge(",
        "Node::separator(false)",
        "Node::separator(state.vertical)",
        ".key(\"card\")",
        ".height(",
    ] {
        assert!(compact.contains(expected), "missing {expected}: {output}");
    }
}

#[test]
fn content_leaf_children_and_unimplemented_semantics_are_diagnosed() {
    for (view, message) in [
        ("<Badge><Text /></Badge>", "does not accept children"),
        (
            "<Separator><Text /></Separator>",
            "does not accept children",
        ),
        ("<Badge @click={save} />", "Unsupported attribute `@click`"),
        (
            "<Separator decorative={false} />",
            "Unsupported attribute `decorative`",
        ),
        ("<Card title=\"Title\" />", "Unsupported attribute `title`"),
    ] {
        let error = compile(&source(view)).unwrap_err();
        assert!(error.message().contains(message), "{error}");
    }
}

#[test]
fn text_edit_modes_lower_to_separate_public_constructors() {
    for (tag, constructor) in [
        ("Input", "input"),
        ("TextInput", "input"),
        ("TextArea", "text_area"),
    ] {
        let output = compile(&source(&format!(
            "<{tag} value={{state.text.clone()}} @input={{changed}} disabled={{state.disabled}} />"
        )))
        .unwrap();
        let compact: String = output.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            compact.contains(&format!("Node::{constructor}(")),
            "{output}"
        );
        let output = compile(&source(&format!("<{tag} model:value={{state.text}} />"))).unwrap();
        let compact: String = output.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(
            compact.contains(&format!("Node::model_{constructor}(")),
            "{output}"
        );
    }
}
