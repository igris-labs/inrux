//! Editor metadata retains authored positions rather than inferring them from formatted output.
use inrux_compiler::{SourceFile, editor_document};

#[test]
fn regions_shield_raw_delimiters_and_recover_incomplete_rust() {
    let text = "<view>\n<Text />\n</view>\n<rust>\nlet example = r#\"\n</rust>\n\"#;\nfn incomplete(\n</rust>\n<style scoped>\n.x { padding: 4px; }\n</style>\n";
    let info = editor_document(SourceFile::new("App.iru", text)).unwrap();
    assert!(text[info.rust].contains("fn incomplete("));
    assert!(info.imports.is_empty());
    assert_eq!(info.styles.len(), 1);
}

#[test]
fn grouped_aliased_imports_have_the_same_bindings_as_compilation() {
    let text = "<view>\n<Dashboard />\n</view>\n<rust>\nuse crate::views::{home as Dashboard, settings};\npub struct State;\n</rust>\n";
    let info = editor_document(SourceFile::new("App.iru", text)).unwrap();
    assert_eq!(info.imports["Dashboard"], ["crate", "views", "home"]);
    assert_eq!(info.imports["settings"], ["crate", "views", "settings"]);
    assert_eq!(&text[info.bindings["Dashboard"].clone()], "Dashboard");
}
