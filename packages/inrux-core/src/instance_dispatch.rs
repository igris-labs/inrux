//! Map child state onto the root coordinator without per-component threads or mutable cross-thread borrows.
use crate::{Context, DispatchError, InstanceHandle, Instances, Sender};
use std::sync::{Arc, Mutex};
impl<S: Clone + Send + Sync + 'static> Sender<S> {
    /// Derive a typed child producer backed by this same bounded application queue.
    pub fn for_instance<C: Clone + Send + Sync + 'static>(
        &self,
        instances: Instances<C>,
        handle: InstanceHandle<C>,
    ) -> Sender<C> {
        let parent = self.clone();
        Sender {
            status: self.status.clone(),
            tx: Arc::new(move |command| {
                if !handle.is_mounted() {
                    return Err(DispatchError::Closed);
                }
                let instances = instances.clone();
                let handle = handle.clone();
                parent.dispatch(move |_, context| {
                    let child = context.for_instance(instances.clone(), handle.clone());
                    instances.update(&handle, |state| command(state, &child));
                })
            }),
        }
    }
}
impl<S: Clone + Send + Sync + 'static> Context<S> {
    /// Route typed task completions through the parent's reserved completion capacity.
    fn for_instance<C: Clone + Send + Sync + 'static>(
        &self,
        instances: Instances<C>,
        handle: InstanceHandle<C>,
    ) -> Context<C> {
        let owner = Some(handle.cancelled.clone());
        let parent_completion = self.completion.clone();
        Context {
            _coordinator: std::cell::Cell::new(()),
            latest: Arc::new(Mutex::new(None)),
            wake: self.wake.clone(),
            cancelled: self.cancelled.clone(),
            observer: Arc::new(Mutex::new(None)),
            tasks: self.tasks.clone(),
            owner,
            completion: Arc::new(move |command| {
                let instances = instances.clone();
                let handle = handle.clone();
                parent_completion(Box::new(move |_, context| {
                    let child = context.for_instance(instances.clone(), handle.clone());
                    instances.update(&handle, |state| command(state, &child));
                }));
            }),
        }
    }
}
