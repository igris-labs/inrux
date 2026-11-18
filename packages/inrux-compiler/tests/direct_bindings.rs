//! Direct names use the same State contract in both setup modes.
use inrux_compiler::{SourceFile, compile, editor_document};
fn source(view: &str, rust: &str) -> String {
    format!("<view>\n{view}\n</view>\n<rust>\n{rust}\n</rust>\n")
}
#[test]
fn direct_nested_models_lower_to_owned_state_paths() {
    let generated = compile(&source("<Input model:value={counter.name} />", "#[derive(Clone)] pub struct State { counter: Counter } pub struct Counter { name: String }")).unwrap();
    let compact = generated.split_whitespace().collect::<String>();
    assert!(compact.contains("set_model_value(&mutstate.counter.name,value"));
}
#[test]
fn props_are_readonly_model_targets() {
    let error = compile(&source(
        "<Input model:value={name} />",
        "pub struct Props { pub name: String }",
    ))
    .unwrap_err();
    assert_eq!(error.code(), "INRUX202");
}
#[test]
fn rust_and_loop_locals_shadow_template_names() {
    let generated = compile(&source(r#"<Column><For each={rows} item="count" key={count}><Text text={format!("{}", count)} /></For><Text text={{let count = 8; format!("{}", count)}} /></Column>"#, "let rows: Vec<u32> = vec![1,2]; let count: u32 = 3;")).unwrap();
    assert!(generated.contains("__inrux_view_each"));
    assert!(!generated.contains("state.count"));
    syn::parse_file(&generated).unwrap();
}
#[test]
fn explicit_state_field_declarations_are_available_to_editor_navigation() {
    let source = source("<Text text={count} />", "#[derive(Clone)] pub struct State { pub count: u32 } pub fn setup() -> State { State { count: 0 } }").replace("<rust>", "<rust explicit>");
    let document = editor_document(SourceFile::new("App.iru", source.as_str())).unwrap();
    assert_eq!(&source[document.bindings["count"].clone()], "count");
}

#[test]
fn block_imports_keep_their_rust_scope() {
    let generated = compile(&source(
        r#"<Text text={{ use crate::constants::name; format!("{}", name) }} />"#,
        "let name: String = String::new();",
    ))
    .unwrap();
    assert!(!generated.contains("state.name"));
}
