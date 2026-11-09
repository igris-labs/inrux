//! Build-time `.iru` compilation into Rust, with original-source syntax diagnostics.
//!
//! Embedded Rust is authored inside `<rust>...</rust>`. This compiler
//! checks framework syntax; rustc subsequently checks Rust expression and property types.
#![deny(missing_docs)]
mod build;
mod codegen;
mod controls;
mod diagnostic;
mod editor;
pub use editor::{EditorDocument, editor_document};
mod format;
mod imports;
pub use build::{BuildOutput, build_app};
pub use format::format_source;
mod sfc;
mod source;
mod style_codegen;
mod view;

pub use diagnostic::{CompileError, Phase, RelatedLocation};
pub use source::{SourceFile, SourcePosition};
use std::{fs, path::Path, sync::Arc};
use view::Element;

/// Compile an in-memory SFC, reporting diagnostics against the virtual `<memory>` file.
///
/// # Errors
/// Returns a structured diagnostic for unsupported or malformed source.
pub fn compile(source: &str) -> Result<String, CompileError> {
    compile_source(SourceFile::new("<memory>", source))
}

/// Compile an owned original source file, preserving source identity and byte ranges.
///
/// # Errors
/// Returns the first block, Rust, template, validation, or stylesheet diagnostic.
/// Rust type errors are reported later by rustc, against generated Rust for now.
pub fn compile_source(source: SourceFile) -> Result<String, CompileError> {
    let source = Arc::new(source);
    let blocks = sfc::parse(&source)?;
    let parse_error = |error: syn::Error, phase, code, block: std::ops::Range<usize>| {
        CompileError::new(
            &source,
            code,
            phase,
            source.parser_range(block, error.span()),
            error.to_string(),
        )
    };
    let rust = syn::parse_file(&source.text()[blocks.rust.body.clone()])
        .map_err(|e| parse_error(e, Phase::Rust, "INRUX101", blocks.rust.body.clone()))?;
    let stateful = rust
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Struct(s) if s.ident == "State"));
    if !stateful
        && !rust
            .items
            .iter()
            .any(|item| matches!(item, syn::Item::Struct(s) if s.ident == "Props"))
    {
        return Err(CompileError::new(
            &source,
            "INRUX102",
            Phase::Rust,
            blocks.rust.opening,
            "Declare State for a stateful root or Props for a stateless component",
        )
        .with_help("Put the component's Rust declarations inside the <rust> block."));
    }
    let tree = syn::parse_str::<Element>(&source.text()[blocks.view.body.clone()])
        .map_err(|e| parse_error(e, Phase::Template, "INRUX201", blocks.view.body.clone()))?;
    tree.validate()
        .map_err(|e| parse_error(e, Phase::Validation, "INRUX202", blocks.view.body.clone()))?;
    imports::validate(&tree, &imports::imports(&rust))
        .map_err(|e| parse_error(e, Phase::Validation, "INRUX203", blocks.view.body.clone()))?;
    let parse_style = |block: Option<sfc::Block>| -> Result<Vec<inrux_style::Rule>, CompileError> {
        if let Some(style) = block {
            inrux_style::parse_css(&source.text()[style.body]).map_err(|error| {
                CompileError::new(&source, "INRUX301", Phase::Style, style.opening, error.to_string())
                    .with_help("Only the documented CSS subset is supported. This location identifies the containing style block.")
            })
        } else {
            Ok(vec![])
        }
    };
    let rules = parse_style(blocks.style)?;
    let global_rules = parse_style(blocks.global_style)?;
    codegen::component(&rust, &tree, stateful, &rules, &global_rules)
        .map_err(|e| parse_error(e, Phase::Validation, "INRUX202", blocks.view.body))
}

/// Compile an SFC file, registering it for Cargo's rebuild tracking.
///
/// # Errors
/// Returns an I/O error for unreadable inputs/unwritable outputs, or an original-source
/// compiler diagnostic. Callers such as build scripts may propagate this error directly.
pub fn compile_file(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = input.as_ref();
    println!("cargo::rerun-if-changed={}", input.display());
    let text = fs::read_to_string(input).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!("Cannot read component file {}: {error}", input.display()),
        )
    })?;
    let rust = compile_source(SourceFile::new(input, text))?;
    fs::write(output, rust)?;
    Ok(())
}

#[cfg(test)]
mod tests;

/// Compile a standalone CSS file's contents into a Rust expression returning typed rules.
/// Use in build.rs with include! to keep parsing and token substitution out of application startup.
///
/// # Errors
/// Returns a style diagnostic for unsupported selectors, declarations, tokens or media queries.
pub fn compile_stylesheet(css: &str) -> Result<String, inrux_style::StyleError> {
    let rules = inrux_style::parse_css(css)?;
    let rules = rules.iter().map(style_codegen::rule_tokens);
    Ok(quote::quote!(vec![#(#rules),*]).to_string())
}
