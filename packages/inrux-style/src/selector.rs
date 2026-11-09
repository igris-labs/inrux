//! Compiled single-element selectors; no string parsing is needed during matching.
use crate::{Part, State, StyleError};

/// A compound type, ID and class selector (without ancestry combinators).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selector {
    /// Case-sensitive framework component name, or None for any type.
    pub kind: Option<String>,
    /// Authored style ID, independent of retention keys and refs.
    pub id: Option<String>,
    /// All of these classes must occur on the node.
    pub classes: Vec<String>,
    /// Optional stable public component part; root selectors leave this unset.
    pub part: Option<Part>,
}
impl Selector {
    /// Parse a bounded compound selector and its optional interaction condition.
    pub fn parse(input: &str) -> Result<(Self, State), StyleError> {
        let (owner, part, suffix) = if let Some((owner, target)) = input.split_once("::part(") {
            let (name, suffix) = target
                .split_once(')')
                .ok_or_else(|| StyleError("unterminated ::part(...) selector".into()))?;
            (owner, Some(Part::parse(name)?), suffix)
        } else {
            let end = input.find(':').unwrap_or(input.len());
            (&input[..end], None, &input[end..])
        };
        let state = match suffix {
            "" => State::Base,
            ":hover" => State::Hover,
            ":focus-visible" => State::Focus,
            ":disabled" => State::Disabled,
            ":checked" => State::Checked,
            ":unchecked" => State::Unchecked,
            _ => {
                return Err(StyleError(
                    "supported states: hover, focus-visible, disabled, checked, unchecked".into(),
                ));
            }
        };
        let input = owner;
        let mut selector = Self {
            part,
            ..Self::default()
        };
        let mut rest = input;
        if let Some(after) = rest.strip_prefix('*') {
            rest = after;
        } else if !rest.starts_with(['.', '#']) {
            let end = rest.find(['.', '#']).unwrap_or(rest.len());
            selector.kind = Some(identifier(&rest[..end])?.into());
            rest = &rest[end..];
        }
        while !rest.is_empty() {
            let prefix = rest.as_bytes()[0];
            if !matches!(prefix, b'.' | b'#') {
                return Err(StyleError(format!("unsupported selector `{input}`")));
            }
            let after = &rest[1..];
            let end = after.find(['.', '#']).unwrap_or(after.len());
            let name = identifier(&after[..end])?.to_owned();
            match prefix {
                b'.' => selector.classes.push(name),
                b'#' if selector.id.is_none() => selector.id = Some(name),
                _ => return Err(StyleError(format!("unsupported selector `{input}`"))),
            }
            rest = &after[end..];
        }
        Ok((selector, state))
    }
    /// CSS specificity ordered by IDs, classes/states and component types.
    pub fn specificity(&self, state: State) -> (usize, usize, usize) {
        (
            usize::from(self.id.is_some()),
            self.classes.len() + usize::from(state != State::Base),
            usize::from(self.kind.is_some()) + usize::from(self.part.is_some()),
        )
    }
    /// Match authored node metadata without allocating or translating native types.
    pub fn matches(&self, kind: &str, id: Option<&str>, classes: &[String]) -> bool {
        self.kind.as_deref().is_none_or(|name| name == kind)
            && self.id.as_deref().is_none_or(|name| Some(name) == id)
            && self.classes.iter().all(|name| classes.contains(name))
    }
}
/// Validate the documented ASCII identifier subset rather than approximating CSS escapes.
fn identifier(name: &str) -> Result<&str, StyleError> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        || name.starts_with(|c: char| c.is_ascii_digit())
    {
        return Err(StyleError(format!(
            "unsupported selector identifier `{name}`; use a type, #id or .class without combinators"
        )));
    }
    Ok(name)
}
