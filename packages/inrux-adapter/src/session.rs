//! One coordinator implementation serves every native host.
use crate::{AdapterError, ViewportState};
use inrux_components::{Component, ComponentHost, Events, Node, Stylesheet};
use inrux_core::{
    Scope, Worker,
    reactive::{Subscription, track},
};
use inrux_style::{CompiledStyles, Rule, Style};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Application ownership and a coalescing mailbox of immutable render descriptions.
/// Dropping the session requests cancellation through Worker without joining user work.
pub struct Session<S: Component> {
    worker: Worker<S>,
    frames: Arc<Mutex<Option<Arc<Node>>>>,
    viewport: ViewportState,
}
impl<S: Component> Session<S> {
    /// Start the coordinator; wake only schedules native work and must never invoke user code.
    pub fn start(
        initial: S,
        scope: Scope,
        rules: Vec<Rule>,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, AdapterError> {
        let viewport = ViewportState::default();
        let scope = scope.provide(viewport.clone());
        let observed_viewport = viewport.clone();
        let base_styles = Arc::new(CompiledStyles::new(rules));
        let cache = Mutex::new((None::<Arc<CompiledStyles>>, base_styles.clone()));
        let worker = Worker::spawn(initial, 64, wake)
            .map_err(|e| AdapterError::Application(e.to_string()))?;
        let frames = Arc::new(Mutex::new(None));
        let slot = frames.clone();
        let events = Events::new(worker.sender());
        let refresh = worker.sender();
        let subscription: Arc<Mutex<Option<Subscription>>> = Arc::new(Mutex::new(None));
        let host = ComponentHost::default();
        let pending = Arc::new(AtomicBool::new(false));
        worker.observe(move |state| {
            pending.store(false, Ordering::Release);
            let coordinator = std::thread::current().id();
            let (frame, dependencies) = track(|| {
                let mut frame = host.render(state, &events, &scope);
                let theme = scope.inject::<Stylesheet>().map(|sheet| sheet.compiled());
                let global = {
                    let mut cached = cache
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let unchanged = match (&cached.0, &theme) {
                        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                        (None, None) => true,
                        _ => false,
                    };
                    if !unchanged {
                        cached.1 = if let Some(theme) = &theme {
                            if base_styles.rules().is_empty() {
                                theme.clone()
                            } else {
                                let mut rules = base_styles.rules().to_vec();
                                rules.extend_from_slice(theme.rules());
                                Arc::new(CompiledStyles::new(rules))
                            }
                        } else {
                            base_styles.clone()
                        };
                        cached.0 = theme;
                    }
                    cached.1.clone()
                };
                frame.resolve_styles(&[&global], observed_viewport.get());
                frame.inherit(&Style::new());
                frame
            });
            let notify = refresh.clone();
            let notified = pending.clone();
            *subscription
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Some(dependencies.subscribe(move || {
                    // Writes in the current handler publish with that command; external writes coalesce.
                    if std::thread::current().id() != coordinator
                        && !notified.swap(true, Ordering::AcqRel)
                        && notify.dispatch(|_, _| {}).is_err()
                    {
                        notified.store(false, Ordering::Release);
                    }
                }));
            // Close the read-to-subscribe race before accepting the published frame.
            if dependencies.is_stale().unwrap_or(true)
                && !pending.swap(true, Ordering::AcqRel)
                && refresh.dispatch(|_, _| {}).is_err()
            {
                pending.store(false, Ordering::Release);
            }
            *slot
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(frame));
        });
        worker
            .sender()
            .dispatch(|_, _| {})
            .map_err(|e| AdapterError::Application(e.to_string()))?;
        Ok(Self {
            worker,
            frames,
            viewport,
        })
    }
    /// Shared viewport reporter for an engine's native resize/layout callback.
    pub fn viewport(&self) -> ViewportState {
        self.viewport.clone()
    }
    /// Consume the newest frame without waiting for the coordinator or cloning its tree.
    pub fn take_frame(&self) -> Result<Option<Arc<Node>>, AdapterError> {
        if self.worker.has_panicked() {
            return Err(AdapterError::Application(
                "Coordinator panicked; close and restart the application".into(),
            ));
        }
        Ok(self.frames.try_lock().ok().and_then(|mut slot| slot.take()))
    }
}
