//! Comment-preserving embedded Rust formatting delegated to the installed rustfmt.
use crate::{SourceFile, sfc};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

/// Format the `<rust>` block while preserving template/style bytes and their comments.
///
/// This first formatter intentionally leaves template and stylesheet formatting unchanged.
/// Requires rustfmt on PATH; invalid or unsupported blocks produce an error without edits.
///
/// # Errors
/// Returns block diagnostics, a missing rustfmt executable, or rustfmt syntax errors.
pub fn format_source(source: SourceFile) -> Result<String, Box<dyn std::error::Error>> {
    let source = Arc::new(source);
    let blocks = sfc::parse(&source)?;
    let original = &source.text()[blocks.rust.body.clone()];
    let implicit = !blocks.rust.explicit && syn::parse_file(original).is_err();
    let input = if implicit {
        format!("fn __inrux_format() {{\n{original}\n}}\n")
    } else {
        original.to_owned()
    };
    let mut process = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    process
        .stdin
        .take()
        .ok_or("Cannot open rustfmt input")?
        .write_all(input.as_bytes())?;
    let output = process.wait_with_output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    let rust = String::from_utf8(output.stdout)?;
    let rust = if implicit {
        let body = rust
            .strip_prefix("fn __inrux_format() {\n")
            .and_then(|body| body.strip_suffix("}\n"))
            .ok_or("Cannot unwrap formatted initialization")?;
        dedent(body)
    } else {
        rust
    };
    let mut text = source.text().to_owned();
    text.replace_range(blocks.rust.body, &format!("\n{rust}"));
    Ok(text)
}

/// Remove wrapper indentation without changing bytes inside multiline Rust literals.
fn dedent(body: &str) -> String {
    let mut offset = 0;
    let literals: Vec<_> = rust_lexer::tokenize(body, rust_lexer::FrontmatterAllowed::No)
        .filter_map(|token| {
            let start = offset;
            offset += token.len as usize;
            matches!(token.kind, rust_lexer::TokenKind::Literal { .. }).then_some(start..offset)
        })
        .collect();
    let mut offset = 0;
    body.split_inclusive('\n')
        .map(|line| {
            let inside_literal = literals
                .iter()
                .any(|literal| offset > literal.start && offset < literal.end);
            offset += line.len();
            if inside_literal {
                line
            } else {
                line.strip_prefix("    ").unwrap_or(line)
            }
        })
        .collect()
}
