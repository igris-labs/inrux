//! Small, quote-aware structural scanner for the explicitly supported stylesheet grammar.
use crate::StyleError;
/// Split a declaration list without treating quoted delimiters as syntax.
pub(crate) fn declarations(input: &str) -> Result<Vec<(&str, &str)>, StyleError> {
    let mut result = Vec::new();
    let mut rest = input;
    while !rest.trim().is_empty() {
        let end = delimiter(rest, ';')?.unwrap_or(rest.len());
        let declaration = rest[..end].trim();
        if !declaration.is_empty() {
            let colon = delimiter(declaration, ':')?
                .ok_or_else(|| StyleError(format!("invalid declaration `{declaration}`")))?;
            result.push((declaration[..colon].trim(), declaration[colon + 1..].trim()));
        }
        rest = if end == rest.len() {
            ""
        } else {
            &rest[end + 1..]
        };
    }
    Ok(result)
}
/// Find a delimiter outside strings and balanced parentheses.
pub(crate) fn delimiter(input: &str, target: char) -> Result<Option<usize>, StyleError> {
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0usize;
    for (i, c) in input.char_indices() {
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
        if c == target && depth == 0 {
            return Ok(Some(i));
        }
        match c {
            '(' => depth += 1,
            ')' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| StyleError("unbalanced CSS parentheses".into()))?
            }
            _ => {}
        }
    }
    if quote.is_some() || depth != 0 || escaped {
        return Err(StyleError("unclosed CSS string or parentheses".into()));
    }
    Ok(None)
}
/// Return a balanced rule body, leaving the next sibling rule untouched.
pub(crate) fn block(input: &str) -> Result<(&str, &str, &str), StyleError> {
    let start = delimiter(input, '{')?
        .ok_or_else(|| StyleError("expected selector { declarations }".into()))?;
    let mut depth = 1usize;
    let mut rest = &input[start + 1..];
    let body = rest;
    while !rest.is_empty() {
        let open = delimiter(rest, '{')?;
        let close = delimiter(rest, '}')?.ok_or_else(|| StyleError("unclosed CSS rule".into()))?;
        if let Some(open) = open.filter(|open| *open < close) {
            depth += 1;
            rest = &rest[open + 1..];
        } else {
            depth -= 1;
            if depth == 0 {
                let end = body.len() - rest.len() + close;
                return Ok((
                    input[..start].trim(),
                    &body[..end],
                    rest[close + 1..].trim(),
                ));
            }
            rest = &rest[close + 1..];
        }
    }
    Err(StyleError("unclosed CSS rule".into()))
}
/// Remove comments outside strings, retaining a separator between adjacent tokens.
pub(crate) fn without_comments(input: &str) -> Result<String, StyleError> {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut quote = None;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if escaped {
            output.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            output.push(c);
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            output.push(c);
            if q == c {
                quote = None;
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            output.push(c);
            continue;
        }
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut closed = false;
            while let Some(c) = chars.next() {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err(StyleError("unclosed CSS comment".into()));
            }
            output.push(' ');
        } else {
            output.push(c);
        }
    }
    Ok(output)
}

/// Split whitespace-separated values while retaining spaces inside color functions and quoted strings.
pub(crate) fn words(input: &str) -> Result<Vec<&str>, StyleError> {
    let mut rest = input.trim();
    let mut values = Vec::new();
    while !rest.is_empty() {
        let mut end = rest.len();
        for space in [' ', '\t', '\n', '\r', '\u{c}'] {
            if let Some(index) = delimiter(rest, space)? {
                end = end.min(index);
            }
        }
        values.push(&rest[..end]);
        rest = rest[end..].trim_start();
    }
    Ok(values)
}
