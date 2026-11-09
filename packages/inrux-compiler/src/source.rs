//! Original source ownership and byte-to-display coordinate mapping.
use std::{
    ops::Range,
    path::{Path, PathBuf},
    sync::Arc,
};

/// An original component file; all diagnostic ranges refer to its UTF-8 bytes.
#[derive(Clone, Debug)]
pub struct SourceFile {
    path: PathBuf,
    text: Arc<str>,
    line_starts: Arc<[usize]>,
}

/// One-based line and Unicode-scalar column, distinct from LSP's UTF-16 positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourcePosition {
    /// One-based source line.
    pub line: usize,
    /// One-based Unicode-scalar column; a tab counts as one scalar.
    pub column: usize,
}

impl SourceFile {
    /// Own source text and index its lines once for subsequent diagnostics.
    pub fn new(path: impl Into<PathBuf>, text: impl Into<Arc<str>>) -> Self {
        let text = text.into();
        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(offset, _)| offset + 1))
            .collect::<Vec<_>>()
            .into();
        Self {
            path: path.into(),
            text,
            line_starts,
        }
    }

    /// Return the original path, including virtual paths such as `<memory>`.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the unchanged UTF-8 input.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Resolve a valid UTF-8 boundary; positions after EOF or inside a character are rejected.
    pub fn position(&self, offset: usize) -> Option<SourcePosition> {
        if !self.text.is_char_boundary(offset) {
            return None;
        }
        let line = self
            .line_starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        let column = self.text[self.line_starts[line]..offset].chars().count() + 1;
        Some(SourcePosition {
            line: line + 1,
            column,
        })
    }

    /// Translate a parser span in a block slice back into the original file.
    pub(crate) fn parser_range(
        &self,
        block: Range<usize>,
        span: proc_macro2::Span,
    ) -> Range<usize> {
        let relative = span.byte_range();
        let start = (block.start + relative.start).min(block.end);
        let end = (block.start + relative.end).min(block.end).max(start);
        // Parser spans originate from the same UTF-8 slice; fallback is honest block-level context.
        if self.text.is_char_boundary(start) && self.text.is_char_boundary(end) {
            start..end
        } else {
            block
        }
    }

    /// Get the containing line's byte range, excluding CRLF/LF terminators.
    pub(crate) fn line_range(&self, offset: usize) -> Range<usize> {
        let line = self
            .line_starts
            .partition_point(|start| *start <= offset)
            .saturating_sub(1);
        let start = self.line_starts[line];
        let end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text.len());
        start..start + self.text[start..end].trim_end_matches(['\r', '\n']).len()
    }
}
