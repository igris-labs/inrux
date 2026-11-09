//! Readable stylesheet validation errors.
use std::fmt;
/// Compiler-facing CSS validation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleError(pub String);
impl fmt::Display for StyleError {
    /// Present the unsupported selector, property, or value to the author.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for StyleError {}
