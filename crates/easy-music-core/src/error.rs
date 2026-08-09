//! Error type shared across the core crate.

use std::fmt;

/// Top-level error for all core operations.
#[derive(Debug)]
pub enum CoreError {
    /// A track or library entry could not be found.
    NotFound(String),
    /// The media backend reported a failure.
    Playback(String),
    /// A generic internal error (database, IO, …).
    Internal(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::NotFound(what) => write!(f, "not found: {what}"),
            CoreError::Playback(msg) => write!(f, "playback error: {msg}"),
            CoreError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}

/// Convenience alias for `Result` values produced by the core crate.
pub type CoreResult<T> = Result<T, CoreError>;
