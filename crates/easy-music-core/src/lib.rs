//! easy-music-core — shared, frontend-agnostic core logic for the EasyMusic
//! desktop application. This crate is compiled into both the Tauri command
//! layer (`src-tauri`) and any future binaries (CLI, workers) so that library
//! management and playback logic live in exactly one place.

pub mod error;
pub mod greeting;
pub mod library;
pub mod playback;

pub use error::{CoreError, CoreResult};
