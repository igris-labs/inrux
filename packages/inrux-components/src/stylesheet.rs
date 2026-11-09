//! Runtime theme replacement uses the same tracked state as component composables.
use inrux_core::reactive::Signal;
use inrux_style::{CompiledStyles, Rule};
use std::sync::Arc;
/// An application-owned global stylesheet, optionally provided through Scope.
#[derive(Clone)]
pub struct Stylesheet {
    rules: Signal<Arc<CompiledStyles>>,
}
impl Stylesheet {
    /// Own a validated CSS stylesheet or a set of typed Rust rules.
    pub fn new(rules: Vec<Rule>) -> Self {
        Self {
            rules: Signal::new(Arc::new(CompiledStyles::new(rules))),
        }
    }
    /// Own an existing immutable index, sharing it across renders and theme toggles.
    pub fn from_compiled(sheet: Arc<CompiledStyles>) -> Self {
        Self {
            rules: Signal::new(sheet),
        }
    }
    /// Read while tracking a view so replacing the theme automatically rebuilds its description.
    pub fn rules(&self) -> Vec<Rule> {
        self.rules.get().rules().to_vec()
    }
    /// Track and borrow the shared index without cloning every rule on every render.
    pub fn compiled(&self) -> Arc<CompiledStyles> {
        self.rules.get()
    }
    /// Replace with an already indexed theme, reusing it across toggles.
    pub fn replace_compiled(&self, sheet: Arc<CompiledStyles>) {
        self.rules.set(sheet);
    }
    /// Replace the theme without recreating the native application or component state.
    pub fn replace(&self, rules: Vec<Rule>) {
        self.rules.set(Arc::new(CompiledStyles::new(rules)));
    }
}
