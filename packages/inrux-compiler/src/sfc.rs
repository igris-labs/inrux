//! Split source blocks only at real lexical boundaries, preserving original byte ranges.
use crate::{CompileError, Phase, SourceFile};
use rust_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use std::{ops::Range, sync::Arc};

/// A source slice and its opening tag, retained for duplicate-block diagnostics.
#[derive(Clone)]
pub(crate) struct Block {
    pub(crate) body: Range<usize>,
    pub(crate) opening: Range<usize>,
}
/// Required view/Rust blocks and optional global/scoped stylesheets.
pub(crate) struct Blocks {
    pub(crate) view: Block,
    pub(crate) rust: Block,
    pub(crate) style: Option<Block>,
    pub(crate) global_style: Option<Block>,
}

/// Discover existing SFC block forms in any order without interpreting Rust syntax.
pub(crate) fn parse(source: &Arc<SourceFile>) -> Result<Blocks, CompileError> {
    let text = source.text();
    let (mut view, mut rust, mut style): (Option<Block>, Option<Block>, Option<Block>) =
        (None, None, None);
    let mut global_style = None;
    let mut offset = 0;
    while offset < text.len() {
        offset += text[offset..].len() - text[offset..].trim_start().len();
        if offset == text.len() {
            break;
        }
        let (opening, closing, slot) = if text[offset..].starts_with("<view>") {
            ("<view>", "</view>", &mut view)
        } else if text[offset..].starts_with("<template>") {
            ("<template>", "</template>", &mut view)
        } else if text[offset..].starts_with("<rust>") {
            ("<rust>", "</rust>", &mut rust)
        } else if text[offset..].starts_with("<style scoped>") {
            ("<style scoped>", "</style>", &mut style)
        } else if text[offset..].starts_with("<style>") {
            ("<style>", "</style>", &mut global_style)
        } else {
            return Err(CompileError::new(
                source,
                "INRUX001",
                Phase::Blocks,
                offset..offset + text[offset..].chars().next().map_or(0, char::len_utf8),
                "Expected <view>, <rust>, <style>, or <style scoped>; <template> is a legacy alias",
            )
            .with_help("Place component content inside a supported block."));
        };
        let opening_range = offset..offset + opening.len();
        if let Some(previous) = slot {
            return Err(CompileError::new(
                source,
                "INRUX002",
                Phase::Blocks,
                opening_range,
                format!("Duplicate {opening} block"),
            )
            .with_related(previous.opening.clone(), "First block declared here")
            .with_help("Keep one block of each supported kind."));
        }
        let body_start = offset + opening.len();
        let end = closing_offset(text, body_start, closing).ok_or_else(|| {
            CompileError::new(source, "INRUX004", Phase::Blocks, opening_range.clone(), format!("Cannot find closing {closing} tag"))
                .with_help(format!("Put {closing} on its own line, outside strings and comments. Check for unterminated literals or comments."))
        })?;
        *slot = Some(Block {
            body: body_start..end,
            opening: opening_range,
        });
        offset = end + closing.len();
    }
    let missing = |tag: &str| {
        CompileError::new(
            source,
            "INRUX003",
            Phase::Blocks,
            text.len()..text.len(),
            format!("Missing {tag} block"),
        )
    };
    Ok(Blocks {
        view: view.ok_or_else(|| missing("<view>"))?,
        rust: rust.ok_or_else(|| missing("<rust>"))?,
        style,
        global_style,
    })
}

/// Accept a delimiter only at a Rust token boundary and on an otherwise empty line.
fn closing_offset(text: &str, start: usize, closing: &str) -> Option<usize> {
    let mut offset = start;
    // The shared lexer shields all Rust literal forms, lifetimes, and nested comments.
    // Template attribute values are Rust-compatible literals/expressions as well.
    for token in tokenize(&text[start..], FrontmatterAllowed::No) {
        if token.kind == TokenKind::Lt
            && text[offset..].starts_with(closing)
            && on_own_line(text, offset, closing.len())
        {
            return Some(offset);
        }
        offset += token.len as usize;
    }
    None
}

/// Enforce the existing own-line delimiter rule while accepting both LF and CRLF.
fn on_own_line(text: &str, offset: usize, length: usize) -> bool {
    let before = text[..offset].rsplit('\n').next().unwrap_or("");
    let after = text[offset + length..].split('\n').next().unwrap_or("");
    before.trim().is_empty() && after.trim().is_empty()
}
