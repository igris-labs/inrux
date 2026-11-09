//! Typed viewport observations and one-shot commands; native wheel input never runs application code.
use crate::views::scrolling::State;
use inrux::{Context, ScrollMetrics, ScrollOffset, ScrollRequest};
/// Issue a new command even if its destination matches an earlier request.
fn request(state: &mut State, y: f32) {
    state.request = ScrollRequest {
        revision: state.request.revision.wrapping_add(1),
        offset: ScrollOffset { x: 0.0, y },
    };
}
/// Move to the start of the outer pane.
pub fn top(state: &mut State, _: &Context<State>) {
    request(state, 0.0);
}
/// Clamp to the actual content end, without querying a native widget handle.
pub fn end(state: &mut State, _: &Context<State>) {
    request(state, f32::MAX);
}
/// Resize without replacing keyed controls or viewport state.
pub fn resize(state: &mut State, _: &Context<State>) {
    state.compact = !state.compact;
}
/// Store an observation without feeding it back as a scroll command.
pub fn observe(state: &mut State, metrics: ScrollMetrics, _: &Context<State>) {
    state.metrics = metrics;
}
/// Ordinary controls inside a viewport use the normal event path.
pub fn increment(state: &mut State, _: &Context<State>) {
    state.count += 1;
}

/// Fill a controlled field with enough text to exercise native horizontal panning.
pub fn long_text(state: &mut State, _: &Context<State>) {
    state.note = format!("START {} END", "Rust workspace name · ".repeat(12));
}
