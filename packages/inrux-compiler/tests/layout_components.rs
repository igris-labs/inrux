//! Built-in layout authoring and useful leaf diagnostics.
use inrux_compiler::compile;

/// Exercise public syntax without relying on private compiler structures.
fn source(view: &str, css: &str) -> String {
    format!(
        "<view>\n{view}\n</view>\n<rust>\npub struct State;\n</rust>\n<style scoped>\n{css}\n</style>\n"
    )
}

#[test]
fn physical_layout_tags_lower_without_component_imports() {
    let output = compile(&source(
        "<Container class=\"panel\"><Box><Row><Text text=\"Title\" /><Spacer /></Row><Center><Text text=\"Body\" /></Center></Box></Container>",
        ".panel { max-width: 800px; flex-grow: 1; margin-inline: auto; }",
    )).unwrap();
    syn::parse_file(&output).unwrap();
    for constructor in ["container", "box_container", "center", "spacer"] {
        assert!(
            output.contains(&format!("Node::{constructor}(")),
            "missing {constructor}"
        );
    }
    assert!(output.contains(".flex_grow("));
    assert!(output.contains(".margin_inline_auto(true)"));
}

#[test]
fn spacer_rejects_content_and_unknown_props_in_authored_source() {
    for (view, diagnostic) in [
        ("<Spacer><Text /></Spacer>", "does not accept children"),
        (
            "<Container mystery={true} />",
            "Unsupported attribute `mystery`",
        ),
    ] {
        let error = compile(&source(view, "")).unwrap_err();
        assert!(error.message().contains(diagnostic), "{error}");
    }
}

/// The disabled input state is available from ordinary markup as well as Node builders.
#[test]
fn input_disabled_attribute_is_lowered_for_model_and_controlled_inputs() {
    for view in [
        "<Input model:value={state.name} :disabled={state.locked} />",
        "<Input value={state.name.clone()} @input={changed} disabled={true} />",
    ] {
        let output = compile(&source(view, "")).unwrap();
        assert!(output.contains(".disabled("));
        syn::parse_file(&output).unwrap();
    }
}

/// Layout props are Rust-typed overrides, while CSS lowers to the same shared values.
#[test]
fn second_group_tags_props_and_css_lower_without_imports() {
    let output=compile(&source("<Grid columns={3}><Flex direction={FlexDirection::Column} wrap={true}><Text /></Flex><AspectRatio ratio={16.0 / 9.0}><Text /></AspectRatio></Grid>",".x { display: grid; grid-template-columns: repeat(4, 1fr); flex-direction: row; aspect-ratio: 2; }")).unwrap();
    syn::parse_file(&output).unwrap();
    for expected in [
        "Node::grid(",
        "Node::flex(",
        "Node::aspect_ratio(",
        ".grid_columns(",
        ".flex_direction(",
        ".aspect_ratio(",
        ".display(",
    ] {
        assert!(output.contains(expected), "missing {expected}");
    }
}

/// Unsupported props are reported against the authored tag rather than dropped.
#[test]
fn second_group_rejects_unimplemented_props() {
    for view in [
        "<Grid spans={2} />",
        "<Flex reverse={true} />",
        "<AspectRatio crop={true} />",
    ] {
        assert!(
            compile(&source(view, ""))
                .unwrap_err()
                .message()
                .contains("Unsupported attribute")
        );
    }
}

/// Authored style IDs, token constants and media conditions survive build-time generation.
#[test]
fn compiled_selectors_and_tokens_emit_typed_rust_without_runtime_parsing() {
    let source = r##"<view>
<Button id="save" class="action" text="Save" />
</view>
<rust>
pub struct Props {}
</rust>
<style scoped>
:root { --gap: 12px; }
Button#save.action { padding: var(--gap); }
@media (max-width: 900px) { .action { padding: 8px; } }
</style>
"##;
    let rust = inrux_compiler::compile(source).unwrap();
    assert!(rust.contains(".id(\"save\")"));
    assert!(rust.contains("::inrux::Selector"));
    assert!(rust.contains("::inrux::Media"));
    assert!(!rust.contains("parse_css"));
    assert!(!rust.contains("var("));
    syn::parse_file(&rust).unwrap();
}
