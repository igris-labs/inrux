//! Retained stateful SFC mounts share the root worker and bounded task executor.
use crate::{Component, Events, Node};
use inrux_core::{InstanceHandle, Instances, Scope};
use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, Weak},
};
type MountKey = (String, TypeId);
struct Entry {
    mount: String,
    value: Arc<dyn Any + Send + Sync>,
    unmount: Box<dyn Fn() + Send + Sync>,
}
impl Drop for Entry {
    /// Invalidate events/tasks before releasing a mounted state generation.
    fn drop(&mut self) {
        (self.unmount)();
    }
}
#[derive(Default)]
struct Storage {
    entries: BTreeMap<MountKey, Entry>,
    visited: BTreeSet<MountKey>,
}
struct Mounted<C> {
    instances: Instances<C>,
    handle: InstanceHandle<C>,
    events: Events<C>,
}
/// Application-owned retained component tree. Each root evaluation sweeps unvisited child mounts.
#[derive(Clone, Default)]
pub struct ComponentHost {
    storage: Arc<Mutex<Storage>>,
}
#[derive(Clone, Default)]
struct RenderPath(String, usize);
#[derive(Clone)]
struct HostRef(Weak<Mutex<Storage>>);
/// Derive an explicit hierarchical identity for a component call or keyed loop iteration.
pub fn component_scope(scope: &Scope, key: impl ToString) -> Scope {
    let parent = scope
        .inject::<RenderPath>()
        .map_or_else(String::new, |path| path.0.clone());
    // Length-prefixing prevents keys containing separators from aliasing a different hierarchy.
    let key = key.to_string();
    let depth = component_depth(scope) + 1;
    scope.provide(RenderPath(format!("{parent}/{}:{key}", key.len()), depth))
}
impl ComponentHost {
    /// Construct a root tree on the coordinator and unmount children absent from its new description.
    pub fn render<S: Component>(&self, state: &S, events: &Events<S>, scope: &Scope) -> Node {
        self.storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .visited
            .clear();
        let scope = scope
            .provide(HostRef(Arc::downgrade(&self.storage)))
            .provide(RenderPath::default());
        let node = state.view(events, &scope);
        let mut rendered = BTreeSet::new();
        collect_mounts(&node, &mut rendered);
        let removed = {
            let mut storage = self
                .storage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let keys: Vec<_> = storage
                .entries
                .keys()
                .filter(|key| {
                    !storage.visited.contains(*key)
                        || !rendered.contains(&storage.entries[*key].mount)
                })
                .cloned()
                .collect();
            keys.into_iter()
                .filter_map(|key| storage.entries.remove(&key))
                .collect::<Vec<_>>()
        };
        // Destructors run outside the host lock, including user-owned component resources.
        drop(removed);
        node
    }
    /// Initialize a child once; subsequent prop updates preserve its existing local state.
    fn child<C: Component, P: Component>(
        &self,
        parent: &Events<P>,
        scope: &Scope,
        setup: impl FnOnce() -> C,
        update_props: impl FnOnce(&mut C),
    ) -> Node {
        let path = scope
            .inject::<RenderPath>()
            .map_or_else(String::new, |path| path.0.clone());
        let key = (path, TypeId::of::<C>());
        let existing = {
            let mut storage = self
                .storage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !storage.visited.insert(key.clone()) {
                return Node::text(
                    "Duplicate component identity; provide distinct sibling keys".into(),
                );
            }
            storage.entries.get(&key).map(|entry| entry.value.clone())
        };
        let mounted = if let Some(existing) = existing {
            match existing.downcast::<Mounted<C>>() {
                Ok(mounted) => mounted,
                Err(_) => {
                    return Node::text(
                        "Component state type does not match its mount identity".into(),
                    );
                }
            }
        } else {
            let instances = Instances::default();
            let handle = instances.mount(setup());
            let events = Events::new(
                parent
                    .sender
                    .for_instance(instances.clone(), handle.clone()),
            );
            let mounted = Arc::new(Mounted {
                instances,
                handle,
                events,
            });
            let registry = mounted.instances.clone();
            let handle = mounted.handle.clone();
            self.storage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entries
                .insert(
                    key,
                    Entry {
                        mount: mounted.handle.to_string(),
                        value: mounted.clone(),
                        unmount: Box::new(move || {
                            registry.unmount(&handle);
                        }),
                    },
                );
            mounted
        };
        mounted.instances.update(&mounted.handle, update_props);
        let mut node = mounted
            .instances
            .with(&mounted.handle, |state| state.view(&mounted.events, scope))
            .unwrap_or_else(|| {
                Node::text("Component was unmounted during view construction".into())
            })
            .key(&mounted.handle);
        let generation = mounted.handle.to_string();
        node.mounts.push(generation.clone());
        node.generation = Some(node.generation.map_or_else(
            || generation.clone(),
            |inner| format!("{generation}/{inner}"),
        ));
        node
    }
}
/// Compiler entry for a stateful child; ordinary Rust users can also compose retained components.
pub fn render_child<C: Component, P: Component>(
    parent: &Events<P>,
    scope: &Scope,
    setup: impl FnOnce() -> C,
    update_props: impl FnOnce(&mut C),
) -> Node {
    match scope.inject::<HostRef>().and_then(|host| host.0.upgrade()) {
        Some(storage) => ComponentHost { storage }.child(parent, scope, setup, update_props),
        None => Node::text(
            "Stateful children require ComponentHost::render (provided by native mount)".into(),
        ),
    }
}

/// Compiler-generated stateless calls also receive stable structural scope identities.
pub fn component_key(scope: &Scope) -> String {
    scope
        .inject::<RenderPath>()
        .map_or_else(String::new, |path| path.0.clone())
}

/// A slot that was composed but never attached must not keep its state or resources mounted.
fn collect_mounts(node: &Node, mounts: &mut BTreeSet<String>) {
    mounts.extend(node.mounts.iter().cloned());
    for child in &node.children {
        collect_mounts(child, mounts);
    }
}

/// Bound compiler-generated recursive component expansion before it can exhaust the stack.
pub fn component_depth(scope: &Scope) -> usize {
    scope.inject::<RenderPath>().map_or(0, |path| path.1)
}
