//! Typed composition examples share the coordinator with retained children.
use crate::views::composition::State;
use inrux::{Color, Context, Scope, Style};
/// Scoped token injected through application and reusable layout boundaries.
#[derive(Clone)]
pub struct Accent(pub Color);
/// Mutate only the first signal/computed pair.
pub fn increment_first(state: &mut State, _: &Context<State>) {
    state.first.count.update(|n| *n += 1);
}
/// Mutate only the second signal/computed pair.
pub fn increment_second(state: &mut State, _: &Context<State>) {
    state.second.count.update(|n| *n += 1);
}
/// Custom model setter and listener publish atomically.
pub fn mark_dirty(state: &mut State, _: &Context<State>) {
    state.saved = false;
    state.message = "Custom model changed; confirmation emits its String payload.".into();
}
/// Receive a typed payload emitted by NameInput.
pub fn confirm_name(state: &mut State, name: String, _: &Context<State>) {
    state.message = format!("Confirmed: {name}");
}
/// Sibling keys preserve each child's note and counter on reorder.
pub fn reverse_rows(state: &mut State, _: &Context<State>) {
    state.rows.reverse();
}
/// Removing a key disposes the child and its queued callbacks.
pub fn remove_row(state: &mut State, _: &Context<State>) {
    if !state.rows.is_empty() {
        state.rows.remove(0);
    }
}
/// Restored keys mount fresh local state, rather than resurrecting disposed instances.
pub fn restore_rows(state: &mut State, _: &Context<State>) {
    for key in [1, 2] {
        if !state.rows.contains(&key) {
            state.rows.push(key);
        }
    }
}
/// Render an injected accent without prop drilling or a renderer import.
pub fn injected_style(scope: &Scope) -> Style {
    Style::new()
        .gap(8.0)
        .padding(8.0)
        .border_width(1.0)
        .border_color(
            scope
                .inject::<Accent>()
                .map(|a| a.0)
                .unwrap_or(Color(29, 112, 98)),
        )
}
