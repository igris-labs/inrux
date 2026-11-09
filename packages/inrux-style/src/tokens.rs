//! Immutable stylesheet-level tokens are substituted before typed declarations are emitted.
use crate::StyleError;
use std::collections::HashMap;
/// Resolve var() references, nested fallbacks and cycles with a bounded recursion budget.
pub(crate) fn resolve(
    value: &str,
    tokens: &HashMap<String, String>,
    stack: &mut Vec<String>,
) -> Result<String, StyleError> {
    resolve_inner(value, tokens, stack, &mut 65_536)
}
/// Bound both expansion work and output size, including repeated references in a token graph.
fn resolve_inner(
    value: &str,
    tokens: &HashMap<String, String>,
    stack: &mut Vec<String>,
    budget: &mut usize,
) -> Result<String, StyleError> {
    if stack.len() > 64 || *budget == 0 {
        return Err(StyleError(
            "CSS token expansion exceeds its nesting/work budget".into(),
        ));
    }
    *budget -= 1;
    let mut output = String::new();
    let mut rest = value;
    while let Some(start) = variable_start(rest) {
        output.push_str(&rest[..start]);
        let body = &rest[start + 4..];
        let mut depth = 1;
        let mut end = None;
        let mut comma = None;
        for (i, c) in body.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i);
                        break;
                    }
                }
                ',' if depth == 1 && comma.is_none() => comma = Some(i),
                _ => {}
            }
        }
        let end = end.ok_or_else(|| StyleError("unclosed var()".into()))?;
        let name = body[..comma.unwrap_or(end)].trim();
        if !name.starts_with("--") || name.len() < 3 {
            return Err(StyleError(format!("invalid token `{name}`")));
        }
        if stack.iter().any(|entry| entry == name) {
            return Err(StyleError(format!(
                "cyclic CSS token `{name}` via {}",
                stack.join(" -> ")
            )));
        }
        stack.push(name.into());
        let replacement = match tokens.get(name) {
            Some(value) => resolve_inner(value, tokens, stack, budget)?,
            None => match comma {
                Some(comma) => resolve_inner(body[comma + 1..end].trim(), tokens, stack, budget)?,
                None => {
                    return Err(StyleError(format!(
                        "undefined CSS token `{name}`; provide a value or var() fallback"
                    )));
                }
            },
        };
        stack.pop();
        if output.len() + replacement.len() > 1_048_576 {
            return Err(StyleError("expanded CSS value exceeds 1 MiB".into()));
        }
        output.push_str(&replacement);
        rest = &body[end + 1..];
    }
    output.push_str(rest);
    Ok(output)
}

/// CSS function-looking text inside a quoted family name remains literal text.
fn variable_start(value: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if q == c {
                quote = None;
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            continue;
        }
        if value[i..].starts_with("var(") {
            return Some(i);
        }
    }
    None
}
