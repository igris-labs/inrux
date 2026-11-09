//! Lexically scoped, typed dependency injection.
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
/// Immutable dependency scope. Descendants shadow ancestors by Rust type.
#[derive(Clone, Default)]
pub struct Scope {
    parent: Option<Arc<Scope>>,
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}
impl Scope {
    /// Create a child scope providing one dependency without mutating the parent.
    pub fn provide<T: Any + Send + Sync>(&self, value: T) -> Self {
        Self {
            parent: Some(Arc::new(self.clone())),
            values: HashMap::from([(
                TypeId::of::<T>(),
                Arc::new(value) as Arc<dyn Any + Send + Sync>,
            )]),
        }
    }
    /// Resolve the nearest provider. Injection itself does not imply reactivity.
    pub fn inject<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.values
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(|v| v.downcast().ok())
            .or_else(|| self.parent.as_ref().and_then(|p| p.inject()))
    }
}
