//! Structured compiler errors with concise source-first text rendering.
use crate::{SourceFile, SourcePosition};
use std::{fmt, ops::Range, path::Path, sync::Arc};

/// Compilation stage that produced a diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// SFC block discovery and delimiters.
    Blocks,
    /// Embedded Rust syntax or missing state/props declaration.
    Rust,
    /// View syntax; the historical variant name is retained for API compatibility.
    Template,
    /// Framework-specific template validation and lowering.
    Validation,
    /// Supported CSS parsing and validation.
    Style,
}

/// Additional source context, for example the first declaration of a duplicate block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedLocation {
    /// Original UTF-8 byte range.
    pub range: Range<usize>,
    /// Explanation of the related source location.
    pub message: String,
}

/// A source-aware failure. Rust type checking still happens separately in rustc.
#[derive(Clone)]
pub struct CompileError {
    source: Arc<SourceFile>,
    code: &'static str,
    phase: Phase,
    range: Range<usize>,
    message: String,
    help: Option<String>,
    related: Vec<RelatedLocation>,
}

impl CompileError {
    /// Build an internal diagnostic from a verified source range.
    pub(crate) fn new(
        source: &Arc<SourceFile>,
        code: &'static str,
        phase: Phase,
        range: Range<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            source: source.clone(),
            code,
            phase,
            range,
            message: message.into(),
            help: None,
            related: vec![],
        }
    }
    /// Attach actionable help without changing the primary source location.
    pub(crate) fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    /// Attach a second original-source location.
    pub(crate) fn with_related(mut self, range: Range<usize>, message: impl Into<String>) -> Self {
        self.related.push(RelatedLocation {
            range,
            message: message.into(),
        });
        self
    }
    /// Stable diagnostic identifier, such as `INRUX201` for template syntax errors.
    pub fn code(&self) -> &'static str {
        self.code
    }
    /// The compiler subsystem that reported the failure.
    pub fn phase(&self) -> Phase {
        self.phase
    }
    /// Source file supplied by the caller; never a generated Rust path for parser failures.
    pub fn source_path(&self) -> &Path {
        self.source.path()
    }
    /// Primary UTF-8 byte range in the original component.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    /// Human-readable explanation, without terminal formatting.
    pub fn message(&self) -> &str {
        &self.message
    }
    /// One-based location; columns count Unicode scalar values, not UTF-16 units.
    pub fn position(&self) -> SourcePosition {
        self.source
            .position(self.range.start)
            .unwrap_or(SourcePosition { line: 1, column: 1 })
    }
    /// Suggested corrective action, when available.
    pub fn help(&self) -> Option<&str> {
        self.help.as_deref()
    }
    /// Related ranges in the same original file.
    pub fn related(&self) -> &[RelatedLocation] {
        &self.related
    }
}

impl fmt::Display for CompileError {
    /// Show one source line and a caret; expand tabs only for terminal alignment.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let position = self.position();
        let line = self.source.line_range(self.range.start);
        let source = self.source.text();
        let caret_start = self.range.start.min(line.end);
        let caret_end = self.range.end.min(line.end).max(caret_start);
        let indentation = unicode_width::UnicodeWidthStr::width(
            source[line.start..caret_start]
                .replace('\t', "    ")
                .as_str(),
        );
        let width =
            unicode_width::UnicodeWidthStr::width(&source[caret_start..caret_end]).clamp(1, 80);
        let gutter = position.line.to_string().len();
        writeln!(f, "error[{}]: {}", self.code, self.message)?;
        writeln!(
            f,
            " --> {}:{}:{}",
            self.source_path().display(),
            position.line,
            position.column
        )?;
        writeln!(f, "{:>gutter$} |", "")?;
        writeln!(
            f,
            "{} | {}",
            position.line,
            source[line].replace('\t', "    ")
        )?;
        write!(
            f,
            "{:>gutter$} | {}{}",
            "",
            " ".repeat(indentation),
            "^".repeat(width)
        )?;
        if let Some(help) = &self.help {
            write!(f, "\nhelp: {help}")?;
        }
        for related in &self.related {
            if let Some(position) = self.source.position(related.range.start) {
                write!(
                    f,
                    "\nnote: {} at {}:{}:{}",
                    related.message,
                    self.source_path().display(),
                    position.line,
                    position.column
                )?;
            }
        }
        Ok(())
    }
}
impl fmt::Debug for CompileError {
    /// Build-script `Result` failures use Debug, so keep that path user-readable too.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl std::error::Error for CompileError {}
