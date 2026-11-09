//! Public part/state selectors lower into typed Rust rather than runtime selector strings.
/// Generated CSS and explicit Rust part styling use the same stable public enum.
#[test]
fn generated_parts_and_states_remain_valid_rust() {
    let source = r#"<view>
<Switch label="Sync" model:checked={state.checked} />
</view>
<rust>
use inrux::prelude::*;
#[derive(Clone, Default)] pub struct State { checked:bool }
</rust>
<style scoped>
Switch::part(track) {width:48px;height:28px;border-radius:14px;}
Switch::part(track):checked {background:#112233;}
Switch::part(thumb) {width:20px;height:20px;}
</style>
"#;
    let output = inrux_compiler::compile(source).unwrap();
    let compact = output.split_whitespace().collect::<String>();
    assert!(compact.contains("Part::Track"));
    assert!(compact.contains("StyleState::Checked"));
    syn::parse_file(&output).unwrap();
}
