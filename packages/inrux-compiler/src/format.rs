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
        .write_all(source.text()[blocks.rust.body.clone()].as_bytes())?;
    let output = process.wait_with_output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    let rust = String::from_utf8(output.stdout)?;
    let mut text = source.text().to_owned();
    text.replace_range(blocks.rust.body, &format!("\n{rust}"));
    Ok(text)
}
