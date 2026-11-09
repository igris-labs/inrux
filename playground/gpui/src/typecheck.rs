//! Generated component props/models/events are checked by real rustc, including failure fixtures.
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
    /// Clean generated metadata even when rustc correctly rejects a fixture.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
/// Type-check generated source against the already-built public facade without another Cargo build.
fn rustc(template: &str, source_type: &str) -> std::process::Output {
    rustc_with_css(template, source_type, "")
}
/// Check stylesheet IR construction in the same real Cargo facade context as generated components.
fn rustc_with_css(template: &str, source_type: &str, css: &str) -> std::process::Output {
    let dependencies = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let library = fs::read_dir(&dependencies)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_name().to_string_lossy().starts_with("libinrux-")
                && entry.path().extension().is_some_and(|ext| ext == "rlib")
        })
        .max_by_key(|entry| entry.metadata().unwrap().modified().unwrap())
        .unwrap()
        .path();
    let directory = std::env::temp_dir().join(format!(
        "inrux typecheck {}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).unwrap();
    let fixture = Fixture(directory);
    let rust = format!(
        r#"
use inrux::prelude::*;
use crate::child as Child;
#[derive(Clone)] pub struct State {{ pub value: {source_type} }}
fn confirmed(_: &mut State, _: String, _: &Context<State>) {{}}
pub mod child {{
    use inrux::prelude::*;
    pub struct Props {{ pub value: String, pub on_update_value: Emit<String>, pub on_confirmed: Emit<String> }}
    pub fn view<P:Component>(_: Props, _: &Events<P>, _: &Scope) -> Node {{ Node::empty() }}
}}
"#
    );
    let source = format!(
        "<template>\n{template}\n</template>\n<rust>\n{rust}\n</rust>\n<style scoped>\n{css}\n</style>\n"
    );
    let generated = inrux_compiler::compile(&source).unwrap();
    let file = fixture.0.join("generated.rs");
    fs::write(&file, generated).unwrap();
    Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name=inrux_fixture",
            "--emit=metadata",
            "--cap-lints=allow",
        ])
        .arg("--extern")
        .arg(format!("inrux={}", library.display()))
        .arg("-L")
        .arg(format!("dependency={}", dependencies.display()))
        .arg("--out-dir")
        .arg(&fixture.0)
        .arg(file)
        .output()
        .unwrap()
}
/// Models support both plain Rust fields and signals; typed emitted payloads infer ordinary handlers.
#[test]
fn component_model_and_payload_types_compile_for_fields_and_signals() {
    for source_type in ["String", "Signal<String>"] {
        let output = rustc(
            "<Child model:value={state.value} @update:value={confirmed} @confirmed={confirmed} />",
            source_type,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
/// No string-based evaluator can silently accept a wrong property or emitted payload type.
#[test]
fn rustc_rejects_unknown_props_and_wrong_payloads() {
    let unknown = rustc(
        "<Child model:value={state.value} @confirmed={confirmed} typo={1} />",
        "String",
    );
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("typo"));
    let wrong = rustc(
        "<Child value={42} on_update_value={Emit::new(|_:String| Ok(()))} @confirmed={confirmed} />",
        "String",
    );
    assert!(!wrong.status.success());
    assert!(String::from_utf8_lossy(&wrong.stderr).contains("String"));
    let payload = rustc(
        "<Child model:value={state.value} @confirmed={|_: &mut State, _: u32, _: &Context<State>| {}} />",
        "String",
    );
    assert!(!payload.status.success());
    assert!(
        String::from_utf8_lossy(&payload.stderr).contains("String"),
        "{}",
        String::from_utf8_lossy(&payload.stderr)
    );
}

/// Built-in component props remain ordinary Rust types, including dynamic separator orientation.
#[test]
fn content_components_typecheck_and_reject_wrong_orientation_types() {
    let valid = rustc(
        "<Card><Badge :text={state.value.clone()} /><Separator vertical={true} /></Card>",
        "String",
    );
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let invalid = rustc("<Separator vertical={state.value.clone()} />", "String");
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("bool"));
}

/// Generated native controls preserve boolean models and numeric range types.
#[test]
fn switch_and_range_props_are_checked_by_rustc() {
    for value_type in ["bool", "Signal<bool>"] {
        let output = rustc(
            r#"<Column><Switch label="Sync" model:checked={state.value} /><Progress label="Work" value={25.0} max={100.0} /><Meter label="Temperature" value={0.0} min={-10.0} max={40.0} /></Column>"#,
            value_type,
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for template in [
        r#"<Switch label="Sync" model:checked={state.value} />"#,
        r#"<Progress label="Work" value={state.value.clone()} max={100.0} />"#,
    ] {
        assert!(!rustc(template, "String").status.success());
    }
}

/// Part selectors and semantic state conditions lower to exported, correctly typed facade values.
#[test]
fn stylesheet_parts_typecheck_against_the_public_facade() {
    let result = rustc_with_css(
        r#"<Switch label="Sync" model:checked={state.value} />"#,
        "bool",
        "Switch {margin:4px 8px;} Switch::part(track) {width:48px;height:28px;} Switch::part(thumb) {width:20px;height:20px;} Switch::part(track):checked {background:oklch(60% 0.1 250 / 50%);}",
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
