//! Error type shared across the core crate.

use std::fmt;

/// Top-level error for all core operations.
#[derive(Debug)]
pub enum CoreError {
    /// A track, playlist, or library entry could not be found.
    NotFound(String),
    /// The media backend reported a failure.
    Playback(String),
    /// A database error (connection, query, constraint).
    Database(String),
    /// An I/O error reading files from disk.
    Io(String),
    /// A metadata / tag parsing error.
    TagParse(String),
    /// Invalid user input (bad filter, empty name, …).
    Invalid(String),
    /// A generic internal error.
    Internal(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::NotFound(what) => write!(f, "not found: {what}"),
            CoreError::Playback(msg) => write!(f, "playback error: {msg}"),
            CoreError::Database(msg) => write!(f, "database error: {msg}"),
            CoreError::Io(msg) => write!(f, "io error: {msg}"),
            CoreError::TagParse(msg) => write!(f, "tag parse error: {msg}"),
            CoreError::Invalid(msg) => write!(f, "invalid: {msg}"),
            CoreError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for CoreError {
    fn from(e: rusqlite::Error) -> Self {
        CoreError::Database(e.to_string())
    }
}

impl From<lofty::error::LoftyError> for CoreError {
    fn from(e: lofty::error::LoftyError) -> Self {
        CoreError::TagParse(e.to_string())
    }
}

/// Convenience alias for `Result` values produced by the core crate.
pub type CoreResult<T> = Result<T, CoreError>;
