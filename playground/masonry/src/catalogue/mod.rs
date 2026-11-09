//! Individual portable component specimens, separate from integration-demo state.
mod content;
mod controls;
mod feedback;
mod layout;
mod metadata;
mod sidebar;
use crate::views::component_preview::State;
use inrux::{Context, Events, Node};
pub use metadata::{EXAMPLES, Example};
pub use sidebar::sidebar;
/// Render only the selected example, keeping native-engine types outside application components.
pub fn preview(state: &State, events: &Events<State>) -> Node {
    use Example::*;
    match state.props.example {
        Button | Input | TextArea | Checkbox | Switch => controls::preview(state, events),
        Progress | Meter => feedback::preview(state, events),
        Text | Badge | Card | Separator => content::preview(state, events),
        _ => layout::preview(state, events),
    }
}
/// Shared preview action changes real view-local state.
pub fn increment(state: &mut State, _: &Context<State>) {
    state.count += 1;
}
/// Reset only this specimen; navigation remounts a different keyed preview instance.
pub fn reset(state: &mut State, _: &Context<State>) {
    state.count = 0;
    state.value = "Inrux Workspace".into();
    state.notes = "First line\nA native multiline editor.\n".into();
    state.checked = false;
    state.disabled = false;
}
