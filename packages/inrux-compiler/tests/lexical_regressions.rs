//! Regression cases for embedded Rust and malformed template input.
use inrux_compiler::compile;

/// Wrap a Rust body in the existing SFC grammar.
fn component(script: &str) -> String {
    format!(
        "<template>\n<Column />\n</template>\n<rust>\n{script}\npub struct State;\npub enum Action {{}}\n</rust>\n"
    )
}

/// A literal containing a delimiter must remain valid embedded Rust.
#[test]
fn closing_tag_inside_raw_rust_string_does_not_end_the_block() {
    let source = component("const CONTENT: &str = r###\"\n</rust>\n\"###;");
    assert!(compile(&source).is_ok());
}

/// Nested Rust comments must shield delimiter text from the SFC scanner.
#[test]
fn closing_tag_inside_nested_comment_does_not_end_the_block() {
    let source = component("/* outer /* nested */\n</rust>\n*/");
    assert!(compile(&source).is_ok());
}

/// Malformed author input must not reach infallible identifier construction.
#[test]
fn invalid_custom_event_is_a_diagnostic_instead_of_a_panic() {
    let source = component("").replace("<Column />", "<Summary @reset:extra={Action::Reset} />");
    let result = std::panic::catch_unwind(|| compile(&source));
    assert!(
        result.is_ok(),
        "the compiler unwound on an invalid event name"
    );
    assert!(result.unwrap().is_err());
}

/// Literal and dynamic spellings must not emit duplicate Rust fields.
#[test]
fn custom_component_duplicate_props_are_rejected_before_rustc() {
    let source = component("").replace("<Column />", "<Summary name=\"one\" :name={state.name} />");
    assert!(compile(&source).is_err());
}

/// Reuse Rust lexical behavior for byte/C literals, escapes, and lifetimes.
#[test]
fn rust_literal_forms_and_lifetimes_keep_their_normal_meaning() {
    let script = r####"
use std::borrow::Cow;
const NORMAL: &str = "</rust> and an escaped quote: \"";
const RAW_BYTES: &[u8] = br##"
</rust>
"##;
const RAW_C: &std::ffi::CStr = cr##"
</rust>
"##;
const RAW_ID: &str = r#"r#type"#;
const QUOTE: char = '\'';
const BYTE: u8 = b'\'';
fn borrow<'a>(value: &'a str) -> Cow<'a, str> { Cow::Borrowed(value) }
// A line comment containing </rust> is not a block boundary.
"####;
    let source = component(script);
    assert!(compile(&source).is_ok());
}

/// Rust expressions inside template attributes keep their literal boundaries.
#[test]
fn raw_template_literal_can_contain_a_template_closing_line() {
    let source = component("").replace("<Column />", "<Text text={r##\"\n</template>\n\"##} />");
    assert!(compile(&source).is_ok());
}

/// Preserve valid stateless components and the existing flexible block order.
#[test]
fn supported_block_orders_and_optional_css_compile() {
    let source = "<rust>\npub struct Props;\n</rust>\n<style scoped>\n.x { color: #ffffff; }\n</style>\n<template>\n<Text text=\"Child\" class=\"x\" />\n</template>\n";
    assert!(compile(source).is_ok());
}

/// An unfinished literal must fail visibly rather than consume the closing delimiter.
#[test]
fn unterminated_rust_literal_returns_an_error_instead_of_hiding_input() {
    let source = component("const BROKEN: &str = r##\"never closed");
    assert!(compile(&source).is_err());
}

/// The published examples and parser must agree on the one supported Rust-block spelling.
#[test]
fn obsolete_rust_block_name_is_rejected_with_the_supported_name() {
    let source = component("")
        .replace("<rust>", "<irs>")
        .replace("</rust>", "</irs>");
    let error = compile(&source).unwrap_err();
    assert_eq!(error.code(), "INRUX001");
    assert!(error.message().contains("<rust>"));
}
