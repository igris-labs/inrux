//! Compile supported CSS into typed selectors, media predicates and declarations.
use crate::{Media, Selector, State, Style, StyleError, syntax, tokens};
use std::collections::HashMap;
/// One compiled selector and typed declaration block; vector order is source order.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    /// Compound selector evaluated against framework metadata.
    pub selector: Selector,
    /// Interaction condition (native support is capability-checked).
    pub state: State,
    /// Viewport predicate; default matches every viewport.
    pub media: Media,
    /// Explicit property overrides supplied by this rule.
    pub style: Style,
}
/// Compile CSS once at build time, or once when a runtime stylesheet is loaded.
/// Tokens declared in :root are immutable sheet constants, not per-element custom properties.
pub fn parse_css(css: &str) -> Result<Vec<Rule>, StyleError> {
    let css = syntax::without_comments(css)?;
    let mut tokens = HashMap::new();
    let mut rest = css.trim();
    while !rest.is_empty() {
        let (selector, body, next) = syntax::block(rest)?;
        if selector == ":root" {
            for (name, value) in syntax::declarations(body)? {
                if !name.starts_with("--") || name.len() < 3 {
                    return Err(StyleError(":root only accepts --token declarations".into()));
                }
                tokens.insert(name.into(), value.into());
            }
        }
        rest = next;
    }
    // Validate even unused constants, so latent cycles cannot depend on component reachability.
    for name in tokens.keys() {
        tokens::resolve(&format!("var({name})"), &tokens, &mut Vec::new())?;
    }
    let mut rules = Vec::new();
    compile(&css, Media::default(), &tokens, &mut rules, 0)?;
    Ok(rules)
}
/// Flatten media blocks in source order, with bounded nesting and typed values.
fn compile(
    css: &str,
    media: Media,
    tokens: &HashMap<String, String>,
    rules: &mut Vec<Rule>,
    depth: usize,
) -> Result<(), StyleError> {
    if depth > 16 {
        return Err(StyleError("CSS media nesting exceeds 16".into()));
    }
    let mut rest = css.trim();
    while !rest.is_empty() {
        let (selector, body, next) = syntax::block(rest)?;
        rest = next;
        if selector == ":root" {
            if depth != 0 {
                return Err(StyleError(
                    "stylesheet tokens must be in top-level :root".into(),
                ));
            }
            continue;
        }
        if let Some(condition) = selector.strip_prefix("@media ") {
            compile(
                body,
                media.intersect(Media::parse(condition)?),
                tokens,
                rules,
                depth + 1,
            )?;
            continue;
        }
        let mut style = Style::new();
        for (property, value) in syntax::declarations(body)? {
            if property.starts_with("--") {
                return Err(StyleError(
                    "custom properties are stylesheet constants; declare them in :root".into(),
                ));
            }
            let value = tokens::resolve(value, tokens, &mut Vec::new())?;
            if value.contains("!important") {
                return Err(StyleError(
                    "!important is not supported; use scope, specificity or explicit Rust style"
                        .into(),
                ));
            }
            style.declaration(property, &value)?;
        }
        for selector in selector.split(',') {
            let (selector, state) = Selector::parse(selector.trim())?;
            if state != State::Base && !style.is_paint_only() {
                return Err(StyleError(
                    "state selectors support color, background and border/radius paint properties only".into(),
                ));
            }
            rules.push(Rule {
                selector,
                state,
                media,
                style: style.clone(),
            });
        }
    }
    Ok(())
}
