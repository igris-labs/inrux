//! Layout changes update styles without changing the retained control's key or mount.
use crate::views::layout_more::State;
use inrux::Context;
/// Switch the main axis while preserving children in source order.
pub fn toggle_direction(state: &mut State, _: &Context<State>) {
    state.column = !state.column;
}
/// Reflow the same children into two or three equal-width tracks.
pub fn toggle_columns(state: &mut State, _: &Context<State>) {
    state.three_columns = !state.three_columns;
}
/// Let an automatic height follow either a square or landscape ratio.
pub fn toggle_ratio(state: &mut State, _: &Context<State>) {
    state.square = !state.square;
}
/// Keep normal event handling available through layout transitions.
pub fn increment(state: &mut State, _: &Context<State>) {
    state.count += 1;
}
