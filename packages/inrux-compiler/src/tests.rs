//! Baseline generated-source regression tests.
use super::*;
/// Wrap baseline template/style cases in the existing component grammar.
fn sfc(template: &str, css: &str) -> String {
    format!(
        "<template>\n{template}\n</template>\n<rust>\npub struct State;\npub enum Action {{}}\n</rust>\n<style scoped>\n{css}\n</style>\n"
    )
}
/// Keep the original valid template wiring intact through the refactor.
#[test]
fn generates_rust_for_binding_events_and_styles() {
    let result = compile(&sfc("<Column class=\"panel\"><Input model:value={state.name} @input={Action::Dirty} /><Button text=\"Save\" @click={Action::Save} /></Column>", ".panel { gap: 16px; }")).unwrap();
    syn::parse_file(&result).unwrap();
    assert!(result.contains("model_versioned"));
}
/// Preserve the original invalid-syntax and CSS-subset rejection cases.
#[test]
fn rejects_malformed_templates_and_unsupported_features() {
    for template in [
        "<Column></Row>",
        "<Text magic={true} />",
        "<Input model:value={other.name} />",
        "<Button><Text /></Button>",
        "<Text text=\"a\" :text={state.name} />",
    ] {
        assert!(compile(&sfc(template, "")).is_err(), "accepted {template}");
    }
    assert!(compile(&sfc("<Column />", ".x { position: sticky; }")).is_err());
}

/// Valid Rust raw identifiers must not panic while constructing a model's update-event identifier.
#[test]
fn raw_identifier_model_fields_lower_without_panicking() {
    let source = "<template>\n<Child model:r#type={state.value} />\n</template>\n<rust>\nuse crate::child as Child; pub struct State { value: String }\n</rust>\n";
    let generated = compile(source).unwrap();
    syn::parse_file(&generated).unwrap();
    assert!(generated.contains("on_update_type"));
}

/// Scroll authoring lowers to shared component/types; geometry styles survive code generation.
#[test]
fn scrolling_authoring_generates_shared_calls_and_clear_event_diagnostics() {
    let source = sfc(
        "<ScrollArea :scroll_to={state.request}><Text text=\"Row\" /></ScrollArea>",
        ".pane { overflow-x: hidden; overflow-y: auto; }",
    );
    let generated = compile(&source).unwrap();
    syn::parse_file(&generated).unwrap();
    assert!(generated.contains("scroll_area"));
    assert!(generated.contains("scroll_to"));
    assert!(generated.contains("Overflow::Auto"));
    let error = compile(&sfc("<ScrollArea @scroll={Action::Changed} />", "")).unwrap_err();
    assert!(error.to_string().contains("typed handler"));
    assert!(compile(&sfc("<Button @scroll={Action::Changed} />", "")).is_err());
}
