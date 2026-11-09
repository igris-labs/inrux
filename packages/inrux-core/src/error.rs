//! Nonblocking command admission errors.
use std::fmt;
/// A bounded queue refuses excess work instead of blocking the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchError {
    /// The bounded action queue has no remaining capacity.
    Full,
    /// The application worker is no longer accepting commands.
    Closed,
}
impl fmt::Display for DispatchError {
    /// Describe overload and shutdown without exposing channel implementation details.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Full => "Application queue is full. Please retry.",
            Self::Closed => "Application worker has stopped.",
        })
    }
}
impl std::error::Error for DispatchError {}
