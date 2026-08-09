//! easy-music-core — shared, frontend-agnostic core logic for the EasyMusic
//! desktop application. This crate is compiled into both the Tauri command
//! layer (`src-tauri`) and any future binaries (CLI, workers) so that library
//! management and playback logic live in exactly one place.

pub mod db;
pub mod error;
pub mod greeting;
pub mod library;
pub mod models;
pub mod playback;
pub mod plugins;
pub mod scanner;

pub use error::{CoreError, CoreResult};
pub use library::LibraryManager;
pub use models::{
    Album, Artist, LibraryMetadata, Playlist, PlaylistWithTracks, ScanResult, Track, TrackFilter,
};
pub use playback::{AudioSink, NullAudioSink, PlaybackEngine, PlaybackState, RepeatMode};
pub use scanner::{scan_directory, ScanOptions};
