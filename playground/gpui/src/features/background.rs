//! View-owned background tasks preserve normal event processing.
use crate::views::background::State;
use inrux::Context;
/// Demonstrate independent updates during worker progress.
pub fn increment(state: &mut State, _: &Context<State>) {
    state.count += 1;
}
/// Publish progress from a cancellable worker while models keep updating on the coordinator.
pub fn start_task(state: &mut State, context: &Context<State>) {
    if state.busy {
        return;
    }
    state.busy = true;
    state.progress.set(0);
    state.message = "Running. Name editing and counters remain available.".into();
    let progress = state.progress.clone();
    match context.spawn_task(
        move |cancel| {
            for step in 1..=60 {
                if cancel.is_cancelled() {
                    return Ok::<_, String>(());
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
                if cancel.is_cancelled() {
                    return Ok(());
                }
                progress.set(step * 100 / 60);
            }
            Ok(())
        },
        |state, result, _| {
            state.busy = false;
            state.task = None;
            match result {
                Ok(()) => {
                    state.completed += 1;
                    state.message = "Finished. Start again or keep editing.".into();
                }
                Err(error) => state.message = format!("Task failed: {error:?}"),
            }
        },
    ) {
        Ok(task) => state.task = Some(task),
        Err(error) => {
            state.busy = false;
            state.message = error.to_string();
        }
    }
}
/// Explicit cancellation prevents a late completion from incrementing the completed counter.
pub fn cancel_task(state: &mut State, _: &Context<State>) {
    if let Some(task) = state.task.take() {
        task.cancel();
    }
    state.busy = false;
    state.message = "Cancelled. Start again whenever you are ready.".into();
}
