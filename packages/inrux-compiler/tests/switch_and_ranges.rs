//! Builtin syntax delegates to typed common constructors and existing model semantics.
use inrux_compiler::compile;
/// New controls require no custom import and reject invalid template contracts early.
#[test]
fn native_control_group_lowers_boolean_models_ranges_and_axis_styles() {
    let source = r##"<view>
<Column><Switch label="Sync" model:checked={state.checked} /><Progress label="Work" value={state.value} max={100.0} /><Meter label="Temperature" value={20.0} min={-10.0} max={40.0} /></Column>
</view>
<rust>
pub struct State { pub checked: bool, pub value: f64 }
</rust>
<style scoped>
Column { row-gap: 12px; column-gap: 8px; accent-color: #123456; }
</style>
"##;
    let output = compile(source).unwrap();
    for constructor in [
        "Node::switch(",
        "Node::progress(",
        "Node::meter(",
        ".row_gap(",
        ".column_gap(",
        ".accent_color(",
    ] {
        assert!(output.contains(constructor), "missing {constructor}");
    }
    syn::parse_file(&output).unwrap();
    for view in [
        "<Progress value={20.0} max={100.0} />",
        "<Meter label=\"X\" value={20.0} max={100.0} />",
        "<Switch label=\"X\" />",
        "<Progress label=\"X\" value={1.0} max={10.0}><Text /></Progress>",
    ] {
        let source = format!("<view>\n{view}\n</view>\n<rust>\npub struct Props {{}}\n</rust>\n");
        assert!(compile(&source).is_err());
    }
}
