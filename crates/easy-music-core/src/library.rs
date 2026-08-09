//! Library management core: tracks, folders, playlists, persistence.

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

/// A single audio track in the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    /// Absolute path on disk.
    pub path: String,
    pub duration_secs: u32,
}

/// In-memory music library. Persistence (SQLite/file scan) is a later step.
#[derive(Debug, Default)]
pub struct LibraryManager {
    tracks: Vec<Track>,
}

impl LibraryManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a track in the library.
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    /// Look up a track by id.
    pub fn get(&self, id: &str) -> CoreResult<&Track> {
        self.tracks
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| CoreError::NotFound(format!("track '{id}'")))
    }

    pub fn all(&self) -> &[Track] {
        &self.tracks
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}
