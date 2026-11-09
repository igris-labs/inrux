//! Preferred view/content syntax shares lowering and diagnostics with its legacy aliases.
use inrux_compiler::{SourceFile, compile, compile_source, format_source};

/// Supply the same typed content contract around either authoring spelling.
fn panel(block: &str, outlet: &str) -> String {
    format!(
        "<{block}>\n<Column><{outlet} /></Column>\n</{block}>\n<rust>\nuse inrux::prelude::*;\npub struct Props {{ pub children: Node }}\n</rust>\n"
    )
}

/// Changing terminology must not alter generated ownership or content behavior.
#[test]
fn default_content_aliases_generate_identical_rust() {
    let expected = compile(&panel("template", "Slot")).unwrap();
    for (block, outlet) in [
        ("view", "Children"),
        ("view", "Slot"),
        ("template", "Children"),
    ] {
        let actual = compile(&panel(block, outlet)).unwrap();
        assert_eq!(actual, expected);
        syn::parse_file(&actual).unwrap();
    }
}

/// Both aliases occupy one UI block and must show both authored locations.
#[test]
fn mixed_ui_blocks_are_duplicates_in_either_order() {
    for (first, second) in [("view", "template"), ("template", "view")] {
        let input = format!(
            "{}<{second}>\n<Column />\n</{second}>\n",
            panel(first, "Children")
        );
        let error = compile(&input).unwrap_err();
        assert_eq!(error.code(), "INRUX002");
        assert_eq!(
            error.range().start,
            input.rfind(&format!("<{second}>")).unwrap()
        );
        assert_eq!(error.related()[0].range, 0..first.len() + 2);
    }
}

/// Compatibility aliases do not permit mismatching opening and closing block names.
#[test]
fn closing_block_must_match_the_opening_spelling() {
    for (opening, wrong) in [("view", "template"), ("template", "view")] {
        let input =
            panel(opening, "Children").replace(&format!("</{opening}>"), &format!("</{wrong}>"));
        let error = compile(&input).unwrap_err();
        assert_eq!(error.code(), "INRUX004");
        assert!(error.message().contains(&format!("</{opening}>")));
    }
}

/// Canonical block splitting still shields raw-string content that resembles a delimiter.
#[test]
fn view_closing_text_inside_a_raw_literal_is_not_a_delimiter() {
    let input = "<view>\n<Text text={r#\"hello\n</view>\nworld\"#} />\n</view>\n<rust>\npub struct Props;\n</rust>\n";
    let output = compile(input).unwrap();
    syn::parse_file(&output).unwrap();
}

/// The new spelling preserves original UTF-8 positions in actionable diagnostics.
#[test]
fn view_errors_keep_original_source_ranges() {
    let input = "<view>\n<Text text=\"🦀\" mystery={true} />\n</view>\n<rust>\npub struct Props;\n</rust>\n";
    let error = compile_source(SourceFile::new("components/我的 Panel.iru", input)).unwrap_err();
    let start = input.find("mystery").unwrap();
    assert_eq!(error.code(), "INRUX202");
    assert_eq!(error.range(), start..start + "mystery".len());
    assert_eq!(error.position().line, 2);
    assert!(error.to_string().contains("components/我的 Panel.iru:2:"));
    let missing = compile("<rust>\npub struct Props;\n</rust>\n").unwrap_err();
    assert!(missing.message().contains("<view>"));
}

/// Formatting embedded Rust preserves either UI spelling and its content comments.
#[test]
fn formatting_preserves_view_aliases_and_content_outlets() {
    for (block, outlet) in [("view", "Children"), ("template", "Slot")] {
        let source = panel(block, outlet);
        let view = source.split("<rust>").next().unwrap();
        let once = format_source(SourceFile::new("Panel.iru", source.as_str())).unwrap();
        let twice = format_source(SourceFile::new("Panel.iru", once.as_str())).unwrap();
        assert!(once.starts_with(view));
        assert_eq!(once, twice);
    }
}
