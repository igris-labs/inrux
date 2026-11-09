//! The compiler follows explicit imports rather than compiling every discovered file.
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    /// Isolate graph tests, including paths containing spaces and Unicode.
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "inrux graph λ {}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(path.join("src/components")).unwrap();
        Self(path)
    }
    /// Write a complete SFC while varying its Rust imports and template.
    fn write(&self, name: &str, template: &str, rust: &str) {
        fs::write(
            self.0.join("src").join(name),
            format!("<template>\n{template}\n</template>\n<rust>\n{rust}\n</rust>\n"),
        )
        .unwrap();
    }
}
impl Drop for Fixture {
    /// Remove generated files even after a failed assertion.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Runtime-selected imported candidates belong to the graph, unreachable broken files do not.
#[test]
fn includes_imported_candidates_and_ignores_unreachable_source() {
    let fixture = Fixture::new();
    fixture.write("App.iru", "<Component :is={if true { First::view(First::Props, scope) } else { Second::view(Second::Props, scope) }} />", "use crate::components::{first as First, second as Second}; pub struct State;");
    fixture.write(
        "components/First.iru",
        "<Text text=\"first\" />",
        "pub struct Props;",
    );
    fixture.write(
        "components/Second.iru",
        "<Text text=\"second\" />",
        "pub struct Props;",
    );
    fs::write(
        fixture.0.join("src/components/Unused.iru"),
        "intentionally invalid",
    )
    .unwrap();
    let result =
        inrux_compiler::build_app(fixture.0.join("src/App.iru"), fixture.0.join("output")).unwrap();
    assert_eq!(result.sources.len(), 3);
    let first = fs::read_to_string(&result.bootstrap).unwrap();
    inrux_compiler::build_app(fixture.0.join("src/App.iru"), fixture.0.join("output")).unwrap();
    assert_eq!(first, fs::read_to_string(&result.bootstrap).unwrap());
    assert!(!first.contains("unused"));
}

/// UTF-8 byte spans remain exact across comments, Unicode and nested generated modules.
#[test]
fn cargo_bootstrap_source_map_round_trips_authored_rust() {
    let fixture = Fixture::new();
    fixture.write(
        "App.iru",
        "<Child />",
        "// λ 😀 kept verbatim\nuse crate::components::child as Child;\npub struct State;\n",
    );
    fixture.write(
        "components/Child.iru",
        "<Text />",
        "// child comment\npub struct Props;\n",
    );
    let result =
        inrux_compiler::build_app(fixture.0.join("src/App.iru"), fixture.0.join("output")).unwrap();
    let bootstrap = fs::read_to_string(result.bootstrap).unwrap();
    let map: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.0.join("output/inrux_app.map.json")).unwrap())
            .unwrap();
    assert_eq!(map["mappings"].as_array().unwrap().len(), 2);
    for entry in map["mappings"].as_array().unwrap() {
        let generated = fs::read_to_string(entry["generated"].as_str().unwrap()).unwrap();
        let start = entry["generatedStart"].as_u64().unwrap() as usize;
        let end = entry["generatedEnd"].as_u64().unwrap() as usize;
        let original = entry["sourceText"].as_str().unwrap();
        let a = entry["originalStart"].as_u64().unwrap() as usize;
        let b = entry["originalEnd"].as_u64().unwrap() as usize;
        assert_eq!(&generated[start..end], &original[a..b]);
        assert!(generated[start..end].contains("//"));
        syn::parse_file(&generated).unwrap();
    }
    syn::parse_file(&bootstrap).unwrap();
}
/// No prelude glob or folder convention implicitly imports application components.
#[test]
fn missing_custom_import_reports_original_file_and_suggestion() {
    let source = inrux_compiler::SourceFile::new(
        "src/App.iru",
        "<template>\n<Counter />\n</template>\n<rust>\nuse inrux::prelude::*; pub struct State;\n</rust>\n",
    );
    let error = inrux_compiler::compile_source(source)
        .unwrap_err()
        .to_string();
    assert!(error.contains("src/App.iru:2:"));
    assert!(error.contains("Cannot find component <Counter>"));
    assert!(error.contains("Counter.iru"));
}
/// A conditional chain is checked structurally instead of falling through to generated Rust errors.
#[test]
fn loops_and_conditionals_are_validated_before_codegen() {
    let fixture = Fixture::new();
    for template in [
        "<Column><Else /></Column>",
        "<For each={0..3} item=\"row\"><Text /></For>",
        "<For each={0..3} item=\"not a name\" key={row}><Text /></For>",
    ] {
        fixture.write("App.iru", template, "pub struct State;");
        assert!(
            inrux_compiler::build_app(fixture.0.join("src/App.iru"), fixture.0.join("output"))
                .is_err()
        );
    }
    fixture.write("App.iru", "<Column><If condition={true}><Text /></If><ElseIf condition={false}><Text /></ElseIf><Else><Text /></Else><For each={0..3} item=\"row\" key={row}><Text /></For></Column>", "pub struct State;");
    inrux_compiler::build_app(fixture.0.join("src/App.iru"), fixture.0.join("output")).unwrap();
}

/// A large source catalogue contributes only its explicitly imported subset to generated code.
#[test]
fn thousand_available_components_compile_only_fifty_imports() {
    let fixture = Fixture::new();
    for index in 0..1000 {
        if index < 50 {
            fixture.write(
                &format!("components/Item{index}.iru"),
                "<Text />",
                "pub struct Props;",
            );
        } else {
            fs::write(
                fixture.0.join(format!("src/components/Item{index}.iru")),
                "unreachable invalid source",
            )
            .unwrap();
        }
    }
    let imports = (0..50)
        .map(|index| format!("item{index} as Item{index}"))
        .collect::<Vec<_>>()
        .join(",");
    fixture.write(
        "App.iru",
        "<Text />",
        &format!("use crate::components::{{{imports}}}; pub struct State;"),
    );
    let graph =
        inrux_compiler::build_app(fixture.0.join("src/App.iru"), fixture.0.join("output")).unwrap();
    assert_eq!(graph.sources.len(), 51);
    assert!(
        !fs::read_to_string(graph.bootstrap)
            .unwrap()
            .contains("item999")
    );
}
