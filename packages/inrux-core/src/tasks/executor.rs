//! A small bounded thread pool; I/O integrations may build async adapters separately.
use std::{
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
type Job = Box<dyn FnOnce() + Send>;
/// Background admission failed without blocking the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskQueueError {
    /// Both running and pending capacity are occupied.
    Full,
    /// The application is shutting down.
    Closed,
}
impl fmt::Display for TaskQueueError {
    /// Give applications an actionable admission error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Full => "Background task queue is full",
            Self::Closed => "Background task executor has stopped",
        })
    }
}
impl std::error::Error for TaskQueueError {}
/// Work failed or unwound; neither failure poisons coordinator state.
#[derive(Debug)]
pub enum TaskError<E> {
    /// The task returned a domain-specific error.
    Failed(E),
    /// A panic was isolated in the task worker.
    Panicked,
}
/// Own the pool; worker threads hold only the receiver/cancellation flag, avoiding ownership cycles.
pub(crate) struct Executor {
    tx: mpsc::SyncSender<Job>,
    cancelled: Arc<AtomicBool>,
}
impl Executor {
    /// Spawn bounded workers; partial initialization is cleaned up through Drop on error.
    pub(crate) fn new(workers: usize, capacity: usize) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::sync_channel::<Job>(capacity);
        let rx = Arc::new(Mutex::new(rx));
        let cancelled = Arc::new(AtomicBool::new(false));
        let pool = Self {
            tx,
            cancelled: cancelled.clone(),
        };
        for index in 0..workers {
            let rx = rx.clone();
            let cancelled = cancelled.clone();
            thread::Builder::new()
                .name(format!("inrux-task-{index}"))
                .spawn(move || {
                    while !cancelled.load(Ordering::Acquire) {
                        // Release the receiver lock before executing work so jobs run concurrently.
                        let next = rx
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .recv_timeout(Duration::from_millis(50));
                        match next {
                            Ok(job) => job(),
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    }
                })?;
        }
        Ok(pool)
    }
    /// Submit without consuming main/coordinator time waiting for capacity.
    pub(crate) fn submit(&self, job: Job) -> Result<(), TaskQueueError> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(TaskQueueError::Closed);
        }
        self.tx.try_send(job).map_err(|e| match e {
            mpsc::TrySendError::Full(_) => TaskQueueError::Full,
            mpsc::TrySendError::Disconnected(_) => TaskQueueError::Closed,
        })
    }
}
impl Drop for Executor {
    /// Never join uncooperative task code during window teardown.
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
