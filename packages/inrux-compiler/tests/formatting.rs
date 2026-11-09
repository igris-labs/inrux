//! Embedded formatting preserves comments and remains stable on repeated formatting.
use inrux_compiler::{SourceFile, format_source};
/// Only the Rust block changes; template/style comments and literal delimiters remain byte-identical.
#[test]
fn rustfmt_is_idempotent_and_preserves_other_blocks_and_comments() {
    let template = "<template>\n// keep this template comment\n<Column><Text text=\"hello\" /></Column>\n</template>\n";
    let style = "<style scoped>\n/* keep this style comment */\n.panel{gap:12px;}\n</style>\n";
    let source = format!(
        "{template}<rust>\n// keep this Rust comment\npub struct State{{pub value:u32}}\n</rust>\n{style}"
    );
    let first = format_source(SourceFile::new("App.iru", source)).unwrap();
    let second = format_source(SourceFile::new("App.iru", first.as_str())).unwrap();
    assert_eq!(first, second);
    assert!(first.starts_with(template));
    assert!(first.ends_with(style));
    assert!(first.contains("// keep this Rust comment"));
}
