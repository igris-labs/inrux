//! Source-coordinate, error-rendering, and public build-helper contracts.
use inrux_compiler::{
    CompileError, Phase, SourceFile, SourcePosition, compile, compile_file, compile_source,
};

/// Construct a component with its template on the second line.
fn component(template: &str) -> String {
    format!(
        "<template>\n{template}\n</template>\n<rust>\npub struct State;\npub enum Action {{}}\n</rust>\n"
    )
}

/// Diagnostics retain original coordinates even before multibyte text.
#[test]
fn semantic_error_preserves_original_utf8_bytes_and_unicode_columns() {
    let input = component("    <Text text=\"你好 🦀\" mystery={true} />");
    let error = compile_source(SourceFile::new("screens/我的 App.iru", input.clone())).unwrap_err();
    let start = input.find("mystery").unwrap();
    let line_start = input.find('\n').unwrap() + 1;
    assert_eq!(error.code(), "INRUX202");
    assert_eq!(error.phase(), Phase::Validation);
    assert_eq!(error.range(), start..start + "mystery".len());
    assert_eq!(
        error.position(),
        SourcePosition {
            line: 2,
            column: input[line_start..start].chars().count() + 1
        }
    );
    assert!(error.to_string().contains("screens/我的 App.iru:2:"));
    assert_eq!(format!("{error:?}"), error.to_string());
}

/// A tag mismatch must identify the authored closing name.
#[test]
fn template_syntax_error_points_to_the_mismatched_closing_tag() {
    let input = component("<Column></Row>");
    let error = compile_source(SourceFile::new("src/App.iru", input.clone())).unwrap_err();
    let start = input.find("Row").unwrap();
    assert_eq!(error.phase(), Phase::Template);
    assert_eq!(error.range(), start..start + 3);
    assert!(error.message().contains("Expected </Column>"));
}

/// Embedded Rust parser ranges must be translated out of the block slice.
#[test]
fn rust_syntax_errors_are_reported_against_the_original_script() {
    let input = component("<Column />").replace("pub struct State;", "pub struct State { name: }");
    let error = compile_source(SourceFile::new("src/App.iru", input.clone())).unwrap_err();
    let close = input.find("name: }").unwrap() + "name: ".len();
    assert_eq!(error.phase(), Phase::Rust);
    assert_eq!(error.range(), close..close + 1);
}

/// Windows newlines must not change logical source columns or leak CRs into display.
#[test]
fn crlf_does_not_shift_source_positions() {
    let input = component("<Text unknown={true} />").replace('\n', "\r\n");
    let error = compile(&input).unwrap_err();
    assert_eq!(error.position(), SourcePosition { line: 2, column: 7 });
    assert_eq!(error.range().start, input.find("unknown").unwrap());
    assert!(!error.to_string().contains('\r'));
}

/// Show both conflicting block declarations to explain duplication.
#[test]
fn duplicate_block_has_a_primary_and_related_location() {
    let input = format!(
        "{}<template>\n<Column />\n</template>\n",
        component("<Column />")
    );
    let error = compile(&input).unwrap_err();
    assert_eq!(error.code(), "INRUX002");
    assert_eq!(error.range().start, input.rfind("<template>").unwrap());
    assert_eq!(error.related()[0].range, 0.."<template>".len());
}

/// Avoid pretending the current CSS parser has property-level ranges.
#[test]
fn stylesheet_error_honestly_identifies_the_containing_block() {
    let input = format!(
        "{}<style scoped>\n.x {{ unsupported: 2px; }}\n</style>\n",
        component("<Column />")
    );
    let error = compile(&input).unwrap_err();
    assert_eq!(error.phase(), Phase::Style);
    assert_eq!(error.range().start, input.find("<style scoped>").unwrap());
    assert!(error.help().unwrap().contains("containing style block"));
}

/// Incomplete source remains navigable at EOF or its unclosed opening tag.
#[test]
fn missing_and_unclosed_blocks_have_actionable_diagnostics() {
    let empty = compile("").unwrap_err();
    assert_eq!(empty.code(), "INRUX003");
    assert_eq!(empty.position(), SourcePosition { line: 1, column: 1 });
    let unclosed = compile("<template>\n<Column />\n").unwrap_err();
    assert_eq!(unclosed.code(), "INRUX004");
    assert!(unclosed.help().unwrap().contains("</template>"));
}

/// Reject invalid byte boundaries while accepting a trailing empty source line.
#[test]
fn source_positions_reject_non_boundaries_and_count_eof_correctly() {
    let source = SourceFile::new("App.iru", "🦀\r\nx\n");
    assert_eq!(source.position(1), None);
    assert_eq!(source.position(100), None);
    assert_eq!(
        source.position(6),
        Some(SourcePosition { line: 2, column: 1 })
    );
    assert_eq!(
        source.position(source.text().len()),
        Some(SourcePosition { line: 3, column: 1 })
    );
}

/// Cargo callers must receive the same structured error as in-memory compilation.
#[test]
fn file_compilation_preserves_the_structured_diagnostic() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unsupported_attribute.iru");
    // Invalid input must fail before attempting to write this deliberately unusable output directory.
    let error = compile_file(&input, std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap_err();
    let diagnostic = error.downcast_ref::<CompileError>().unwrap();
    assert_eq!(diagnostic.source_path(), input);
    assert_eq!(diagnostic.position().line, 2);
    assert!(format!("{error:?}").contains("error[INRUX202]"));
}

/// Code generation must remain reproducible and parseable without runtime parsing.
#[test]
fn output_is_deterministic_and_valid_rust() {
    let input =
        component("<Column><Text text=\"Hello\" /><Input model:value={state.name} /></Column>");
    let first = compile(&input).unwrap();
    assert_eq!(first, compile(&input).unwrap());
    syn::parse_file(&first).unwrap();
}

/// Invalid names and alias collisions are parser failures, not compiler crashes.
#[test]
fn malformed_component_names_and_events_do_not_unwind() {
    for template in [
        "<Self />",
        "<Type />",
        "<Summary @reset:bad={Action::Reset} />",
        "<Summary @reset={a} :on_reset={b} />",
        "<Summary model:value={state.name} />",
    ] {
        let result = std::panic::catch_unwind(|| compile(&component(template)));
        assert!(result.is_ok(), "compiler unwound on {template}");
        assert!(result.unwrap().is_err(), "accepted {template}");
    }
}

/// Limit author nesting before recursive parsing/generation exhausts the stack.
#[test]
fn deeply_nested_templates_return_a_diagnostic() {
    let template = format!(
        "{}<Text />{}",
        "<Column>".repeat(150),
        "</Column>".repeat(150)
    );
    let error = compile(&component(&template)).unwrap_err();
    assert!(error.message().contains("nesting"));
}

/// Keep the human-facing build-script format stable and independent of generated file paths.
#[test]
fn diagnostic_display_is_a_short_original_source_excerpt() {
    let input = component("<Text unknown={true} />");
    let error = compile_source(SourceFile::new("src/App.iru", input)).unwrap_err();
    assert_eq!(
        error.to_string(),
        "error[INRUX202]: Unsupported attribute `unknown` on <Text>\n --> src/App.iru:2:7\n  |\n2 | <Text unknown={true} />\n  |       ^^^^^^^"
    );
}

/// Display-cell alignment is separate from source columns for tabs, CJK, and emoji.
#[test]
fn caret_alignment_handles_wide_text_and_multi_digit_line_numbers() {
    let input = format!(
        "{}{}",
        "\n".repeat(10),
        component("\t<Text text=\"你好 🦀\" unknown={true} />")
    );
    let error = compile(&input).unwrap_err();
    assert_eq!(
        error.position(),
        SourcePosition {
            line: 12,
            column: 20
        }
    );
    assert_eq!(
        error.to_string().lines().last().unwrap(),
        format!("   | {}^^^^^^^", " ".repeat(25))
    );
}
