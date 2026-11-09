//! Immutable rule index shared by coordinator renders, with one anchor per rule.
use crate::{Rule, Viewport};
use std::collections::HashMap;
/// An indexed stylesheet; constructing it never involves native engine types.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompiledStyles {
    rules: Vec<Rule>,
    ids: HashMap<String, Vec<usize>>,
    classes: HashMap<String, Vec<usize>>,
    kinds: HashMap<String, Vec<usize>>,
    universal: Vec<usize>,
}
impl CompiledStyles {
    /// Index an already typed rule set once, preserving source order for cascade ties.
    pub fn new(rules: Vec<Rule>) -> Self {
        let mut result = Self {
            rules,
            ..Self::default()
        };
        for (i, rule) in result.rules.iter().enumerate() {
            if let Some(id) = &rule.selector.id {
                result.ids.entry(id.clone()).or_default().push(i);
            } else if let Some(class) = rule.selector.classes.first() {
                result.classes.entry(class.clone()).or_default().push(i);
            } else if let Some(kind) = &rule.selector.kind {
                result.kinds.entry(kind.clone()).or_default().push(i);
            } else {
                result.universal.push(i);
            }
        }
        result
    }
    /// Access immutable typed rules for tooling and compatibility APIs.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
    /// Visit candidate rules in specificity/source order, allocating only the matched-index buffer.
    pub fn matching(
        &self,
        kind: &str,
        id: Option<&str>,
        classes: &[String],
        viewport: Viewport,
    ) -> impl Iterator<Item = &Rule> {
        let mut candidates = self.universal.clone();
        if let Some(ids) = id.and_then(|id| self.ids.get(id)) {
            candidates.extend(ids);
        }
        if let Some(ids) = self.kinds.get(kind) {
            candidates.extend(ids);
        }
        for class in classes {
            if let Some(ids) = self.classes.get(class) {
                candidates.extend(ids);
            }
        }
        candidates.retain(|i| {
            let rule = &self.rules[*i];
            rule.media.matches(viewport) && rule.selector.matches(kind, id, classes)
        });
        candidates.sort_unstable_by_key(|i| {
            (
                self.rules[*i].selector.specificity(self.rules[*i].state),
                *i,
            )
        });
        candidates.dedup();
        candidates.into_iter().map(|i| &self.rules[i])
    }
}
