//! Implicit initialization stays a compile-time transformation with reversible editor ranges.
use inrux_compiler::{SourceFile, compile, editor_document, format_source};

fn component(rust: &str) -> String {
    format!("<view>\n<Text text={{state.count}} />\n</view>\n<rust>\n{rust}\n</rust>\n")
}

#[test]
fn typed_bindings_generate_state_and_setup_while_helpers_remain_local() {
    let generated = compile(&component("use inrux::prelude::*;\nlet helper = 2;\nlet count: u32 = helper + 1;\nfn increment(state: &mut State, _: &Context<State>) { state.count += 1; }")).unwrap();
    let module = syn::parse_file(&generated).unwrap();
    let state = module
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Struct(item) if item.ident == "State" => Some(item),
            _ => None,
        })
        .unwrap();
    let fields: Vec<_> = state
        .fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(fields, ["count"]);
    assert!(generated.contains("pub fn setup() -> State"));
    assert!(generated.contains("let helper = 2"));
}

#[test]
fn props_are_initialized_once_and_remain_available_to_the_view() {
    let generated = compile(&component(
        "#[derive(Clone)] pub struct Props { pub initial: u32 }\nlet count: u32 = props.initial; ",
    ))
    .unwrap();
    assert!(generated.contains("pub props: Props"));
    assert!(generated.contains("setup(props: Props)"));
    assert!(generated.contains("let props = &self.props"));
    assert!(generated.contains("state.props = props"));
}

#[test]
fn initialization_errors_identify_the_authored_binding() {
    for rust in [
        "let count: u32 = 0; let count: u32 = 1;",
        "let props: u32 = 0;",
        "let (first, second): (u32, u32) = (1, 2);",
        "let count: u32;",
        "let count: u32 = 0; pub fn setup() -> State { todo!() }",
        "let count: u32 = 0; count",
    ] {
        let error = compile(&component(rust)).unwrap_err();
        assert_eq!(error.code(), "INRUX101");
    }
}

#[test]
fn projections_preserve_unicode_source_and_lift_imports_for_navigation() {
    let source = component(
        "use crate::components::counter as Counter;\nlet helper = \"日本語\";\nlet count: String = helper.to_owned();",
    );
    let info = editor_document(SourceFile::new("App.iru", source.as_str())).unwrap();
    assert_eq!(&source[info.bindings["count"].clone()], "count");
    assert_eq!(&source[info.bindings["Counter"].clone()], "Counter");
    let projection = info.projection.unwrap();
    let code = projection["code"].as_str().unwrap();
    syn::parse_file(code).unwrap();
    let original = &source[info.rust];
    for mapping in projection["mappings"].as_array().unwrap() {
        let range = |prefix: &str| {
            mapping[format!("{prefix}Start")].as_u64().unwrap() as usize
                ..mapping[format!("{prefix}End")].as_u64().unwrap() as usize
        };
        assert_eq!(&original[range("original")], &code[range("generated")]);
    }
}

#[test]
fn implicit_formatting_is_idempotent_and_keeps_raw_literal_indentation() {
    let source = component(
        "// preserve comment\nlet count:u32=1;\nlet text: &str = r#\"first\n    four spaces\nlast\"#;",
    );
    let once = format_source(SourceFile::new("App.iru", source)).unwrap();
    let twice = format_source(SourceFile::new("App.iru", once.as_str())).unwrap();
    assert_eq!(once, twice);
    assert!(once.contains("let count: u32 = 1;"));
    assert!(once.contains("r#\"first\n    four spaces\nlast\"#"));
    assert!(once.contains("// preserve comment"));
}

#[test]
fn explicit_state_can_use_implicit_initialization_without_a_user_setup_function() {
    let generated = compile(&component("#[derive(Clone)] pub struct State { pub count: u32 }\nlet initial = 4;\nState { count: initial }")).unwrap();
    syn::parse_file(&generated).unwrap();
    assert_eq!(generated.matches("pub struct State").count(), 1);
    assert_eq!(generated.matches("pub fn setup").count(), 1);
}

#[test]
fn conditional_initialization_and_cfg_fields_lower_as_ordinary_rust() {
    let generated = compile(&component("#[cfg(any())] let optional: bool = false;\nlet count: u32 = 1;\nif count > 0 { println!(\"ready\"); }")).unwrap();
    syn::parse_file(&generated).unwrap();
    assert_eq!(generated.matches("#[cfg(any())]").count(), 3);
}

#[test]
fn manual_setup_requires_the_explicit_rust_tag() {
    let body = "#[derive(Clone)] pub struct State { pub count: u32 }\npub fn setup() -> State { State { count: 1 } }";
    let implicit = component(body);
    let error = compile(&implicit).unwrap_err();
    assert!(error.to_string().contains("<rust explicit>"));
    let explicit = implicit.replace("<rust>", "<rust explicit>");
    let generated = compile(&explicit).unwrap();
    assert_eq!(generated.matches("pub fn setup").count(), 1);
    assert_eq!(generated.matches("pub struct State").count(), 1);
    let info = editor_document(SourceFile::new("App.iru", explicit.as_str())).unwrap();
    assert!(explicit[info.rust].contains("pub fn setup"));
    let once = format_source(SourceFile::new("App.iru", explicit)).unwrap();
    let twice = format_source(SourceFile::new("App.iru", once.as_str())).unwrap();
    assert_eq!(once, twice);
    assert!(once.contains("<rust explicit>"));
}

#[test]
fn explicit_blocks_require_a_module_with_state_and_setup() {
    for body in [
        "let count: u32 = 0;",
        "#[derive(Clone)] pub struct State { pub count: u32 }",
        "pub fn setup() -> u32 { 0 }",
    ] {
        let error = compile(&component(body).replace("<rust>", "<rust explicit>")).unwrap_err();
        assert_eq!(error.code(), "INRUX101");
    }
}

#[test]
fn rust_block_modes_cannot_be_duplicated() {
    let source = format!(
        "{}<rust explicit>\npub struct State; pub fn setup() -> State {{ State }}\n</rust>\n",
        component("let count: u32 = 0;")
    );
    let error = compile(&source).unwrap_err();
    assert_eq!(error.code(), "INRUX002");
}
