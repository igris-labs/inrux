//! Serialized application state and immutable publication.
use crate::{
    DispatchError,
    tasks::{Cancellation, Executor, TaskError, TaskHandle, TaskQueueError},
};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, TrySendError},
    },
    thread,
    time::Duration,
};
pub(crate) type Command<S> = Box<dyn FnOnce(&mut S, &Context<S>) + Send>;
type Producer<S> = Arc<dyn Fn(Command<S>) -> Result<(), DispatchError> + Send + Sync>;
type Completion<S> = Arc<dyn Fn(Command<S>) + Send + Sync>;
type Snapshot<S> = Arc<Mutex<Option<Arc<S>>>>;
type Observer<S> = Arc<Mutex<Option<Arc<dyn Fn(&S) + Send + Sync>>>>;

/// Context available to worker actions, including intermediate snapshot publication.
pub struct Context<S> {
    // The context moves into its coordinator once, but cannot be borrowed by a
    // scoped background thread that would publish state outside command ordering.
    pub(crate) _coordinator: std::cell::Cell<()>,
    pub(crate) latest: Snapshot<S>,
    pub(crate) wake: Arc<dyn Fn() + Send + Sync>,
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) observer: Observer<S>,
    pub(crate) tasks: Arc<Executor>,
    pub(crate) completion: Completion<S>,
    pub(crate) owner: Option<Arc<AtomicBool>>,
}
impl<S: Clone> Context<S> {
    /// Publish intermediate state, e.g. before starting a long operation.
    pub fn publish(&self, state: &S) {
        let snapshot = Arc::new(state.clone());
        if let Ok(mut latest) = self.latest.lock() {
            *latest = Some(snapshot);
        }
        let observer = self
            .observer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(observer) = observer {
            observer(state);
        }
        (self.wake)();
    }
    /// Long operations should cooperate with window shutdown.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
            || self
                .owner
                .as_ref()
                .is_some_and(|owner| owner.load(Ordering::Acquire))
    }
}

impl<S: Clone + Send + Sync + 'static> Context<S> {
    /// Run owned work off the coordinator and fairly apply its typed result later.
    pub fn spawn_task<T: Send + 'static, E: Send + 'static>(
        &self,
        work: impl FnOnce(Cancellation) -> Result<T, E> + Send + 'static,
        apply: impl FnOnce(&mut S, Result<T, TaskError<E>>, &Context<S>) + Send + 'static,
    ) -> Result<TaskHandle, TaskQueueError> {
        self.spawn_task_inner(None, work, apply)
    }
    /// Run work owned by a mount generation; unmount discards late results automatically.
    pub fn spawn_owned_task<I, T: Send + 'static, E: Send + 'static>(
        &self,
        owner: &crate::InstanceHandle<I>,
        work: impl FnOnce(Cancellation) -> Result<T, E> + Send + 'static,
        apply: impl FnOnce(&mut S, Result<T, TaskError<E>>, &Context<S>) + Send + 'static,
    ) -> Result<TaskHandle, TaskQueueError> {
        self.spawn_task_inner(Some(owner.cancelled.clone()), work, apply)
    }
    /// Share admission, panic isolation, and typed delivery across root-owned and instance-owned tasks.
    fn spawn_task_inner<T: Send + 'static, E: Send + 'static>(
        &self,
        owner: Option<Arc<AtomicBool>>,
        work: impl FnOnce(Cancellation) -> Result<T, E> + Send + 'static,
        apply: impl FnOnce(&mut S, Result<T, TaskError<E>>, &Context<S>) + Send + 'static,
    ) -> Result<TaskHandle, TaskQueueError> {
        let token = Cancellation {
            parent: self.owner.clone(),
            owner,
            application: self.cancelled.clone(),
            local: Arc::new(AtomicBool::new(false)),
        };
        if token.is_cancelled() {
            return Err(TaskQueueError::Closed);
        }
        let handle = TaskHandle {
            token: token.clone(),
        };
        let completion = self.completion.clone();
        self.tasks.submit(Box::new(move || {
            if token.is_cancelled() {
                return;
            }
            let result = catch_unwind(AssertUnwindSafe(|| work(token.clone())))
                .map_err(|_| TaskError::Panicked)
                .and_then(|result| result.map_err(TaskError::Failed));
            if !token.is_cancelled() {
                // Completion has its own bounded queue; background workers may wait, the UI never does.
                completion(Box::new(move |state, context| {
                    if !token.is_cancelled() {
                        apply(state, result, context);
                    }
                }));
            }
        }))?;
        Ok(handle)
    }
}

/// Nonblocking producer for commands that execute exclusively on the worker.
pub struct Sender<S> {
    pub(crate) tx: Producer<S>,
    pub(crate) status: Arc<AtomicU8>,
}
impl<S> Clone for Sender<S> {
    /// Share the same bounded producer and application status.
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            status: self.status.clone(),
        }
    }
}
impl<S> Sender<S> {
    /// Queue a state mutation. No application code executes on the calling thread.
    pub fn dispatch(
        &self,
        f: impl FnOnce(&mut S, &Context<S>) + Send + 'static,
    ) -> Result<(), DispatchError> {
        if self.status.load(Ordering::Acquire) != 0 {
            return Err(DispatchError::Closed);
        }
        (self.tx)(Box::new(f))
    }
}

/// Main-thread handle. State snapshots are immutable and reads never wait on the worker.
pub struct Worker<S> {
    sender: Sender<S>,
    latest: Snapshot<S>,
    cancelled: Arc<AtomicBool>,
    observer: Observer<S>,
}
impl<S: Clone + Send + Sync + 'static> Worker<S> {
    /// Start a named worker with a bounded action queue and a repaint notification.
    pub fn spawn(
        initial: S,
        capacity: usize,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::sync_channel::<Command<S>>(capacity);
        let (completion, completed) = mpsc::sync_channel::<Command<S>>(18);
        let observer: Observer<S> = Arc::new(Mutex::new(None));
        let latest = Arc::new(Mutex::new(Some(Arc::new(initial.clone()))));
        let cancelled = Arc::new(AtomicBool::new(false));
        let status = Arc::new(AtomicU8::new(0));
        let context = Context {
            _coordinator: std::cell::Cell::new(()),
            latest: latest.clone(),
            wake: Arc::new(wake),
            cancelled: cancelled.clone(),
            observer: observer.clone(),
            tasks: Arc::new(Executor::new(2, 16)?),
            completion: Arc::new(move |command| {
                let _ = completion.send(command);
            }),
            owner: None,
        };
        let worker_status = status.clone();
        thread::Builder::new()
            .name("inrux-application".into())
            .spawn(move || {
                let mut state = initial;
                let result = catch_unwind(AssertUnwindSafe(|| {
                    while !context.is_cancelled() {
                        // Drain one completion per turn even when normal events continuously arrive.
                        if let Ok(command) = completed.try_recv() {
                            command(&mut state, &context);
                            context.publish(&state);
                        }
                        match rx.recv_timeout(Duration::from_millis(50)) {
                            Ok(command) => {
                                command(&mut state, &context);
                                context.publish(&state);
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => continue,
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    }
                }));
                // A stopped coordinator cannot apply results, including after a handler panic.
                context.cancelled.store(true, Ordering::Release);
                worker_status.store(if result.is_err() { 2 } else { 1 }, Ordering::Release);
                (context.wake)();
            })?;
        Ok(Self {
            sender: Sender {
                tx: Arc::new(move |command| {
                    tx.try_send(command).map_err(|error| match error {
                        TrySendError::Full(_) => DispatchError::Full,
                        TrySendError::Disconnected(_) => DispatchError::Closed,
                    })
                }),
                status,
            },
            latest,
            cancelled,
            observer,
        })
    }
    /// Observe committed state on the coordinator, never on the calling/UI thread.
    pub fn observe(&self, observer: impl Fn(&S) + Send + Sync + 'static) {
        *self
            .observer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(observer));
    }
    /// Get a command producer for event handlers.
    pub fn sender(&self) -> Sender<S> {
        self.sender.clone()
    }
    /// Take the newest snapshot, or keep rendering the previous one if the slot is busy.
    pub fn take_snapshot(&self) -> Option<Arc<S>> {
        self.latest.try_lock().ok()?.take()
    }
    /// Whether an action panicked. A panicked worker is stopped, never reused.
    pub fn has_panicked(&self) -> bool {
        self.sender.status.load(Ordering::Acquire) == 2
    }
}
impl<S> Drop for Worker<S> {
    /// Request cancellation without joining potentially uncooperative user code.
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
        self.sender.status.store(1, Ordering::Release);
        // Do not join here: an uncooperative handler must not block window teardown.
    }
}
