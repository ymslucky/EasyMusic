//! Data models for the music library — all serde-serializable so the Tauri
//! frontend receives clean JSON via `invoke()`.

use serde::{Deserialize, Serialize};

/// A single audio track in the library.
///
/// `artist` and `album` are denormalized strings for convenience on the
/// frontend; the DB normalizes them into separate tables for aggregation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    /// Stable unique identifier (UUID v4).
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub genre: Option<String>,
    /// Absolute path on disk.
    pub path: String,
    /// Duration in seconds (0 if unknown).
    pub duration_secs: u32,
    /// 1-based track number within the album, if tagged.
    pub track_number: Option<u32>,
    /// Release year, if tagged.
    pub year: Option<u32>,
    /// File extension (e.g. `"mp3"`, `"flac"`).
    pub file_format: Option<String>,
}

/// An album — a logical grouping of tracks by title + artist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub track_count: u32,
}

/// An artist — derived from distinct `track.artist` values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub album_count: u32,
    pub track_count: u32,
}

/// A user-created playlist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub track_count: u32,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
}

/// A playlist with its full track list — returned by `get_playlist`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistWithTracks {
    pub playlist: Playlist,
    pub tracks: Vec<Track>,
}

/// Aggregated statistics about the entire library.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryMetadata {
    pub total_tracks: u32,
    pub total_albums: u32,
    pub total_artists: u32,
    pub total_playlists: u32,
    /// Total playtime across all tracks, in seconds.
    pub total_duration_secs: u64,
    /// ISO-8601 timestamp of the last successful scan, if any.
    pub last_scanned: Option<String>,
}

/// Result of a library scan — returned to the frontend for feedback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scanned_files: u32,
    pub added: u32,
    pub updated: u32,
    pub skipped: u32,
    pub errors: u32,
}

/// Filtering criteria for `LibraryManager::filter_tracks`.
///
/// All fields are optional; `None` means "no constraint". Multiple fields
/// are AND-ed together.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackFilter {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    /// Minimum duration in seconds (inclusive).
    pub min_duration_secs: Option<u32>,
    /// Maximum duration in seconds (inclusive).
    pub max_duration_secs: Option<u32>,
}
