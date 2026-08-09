//! Library management — the high-level facade the Tauri command layer calls.
//!
//! Wraps [`Database`] with a friendlier API and coordinates scanning +
//! indexing. All methods are synchronous because `rusqlite` is sync; the
//! Tauri commands mark themselves `async` and offload with `tokio::task::spawn_blocking`.

use std::path::Path;

use crate::db::Database;
use crate::error::CoreResult;
use crate::models::{
    Album, Artist, LibraryMetadata, Playlist, PlaylistWithTracks, ScanResult, Track, TrackFilter,
};
use crate::scanner::{scan_directory, ScanOptions};

/// The public library manager. Owns a SQLite database handle.
pub struct LibraryManager {
    db: Database,
}

impl LibraryManager {
    /// Open a library backed by a file on disk.
    pub fn open(db_path: &str) -> CoreResult<Self> {
        Ok(Self {
            db: Database::open(db_path)?,
        })
    }

    /// Open a library backed by an in-memory DB (tests).
    pub fn open_memory() -> CoreResult<Self> {
        Ok(Self {
            db: Database::open_memory()?,
        })
    }

    /// Direct access to the underlying DB — used by the Tauri command layer
    /// for operations that don't need the convenience wrapper.
    pub fn db(&self) -> &Database {
        &self.db
    }

    // -- scanning -------------------------------------------------------

    /// Scan `root`, upsert all discovered tracks, and record the scan time.
    pub fn scan_directory(&self, root: &Path) -> CoreResult<ScanResult> {
        let tracks = scan_directory(&ScanOptions::new(root))?;
        let mut result = self.db.upsert_tracks_batch(&tracks)?;
        let now = chrono::Utc::now().to_rfc3339();
        self.db.set_last_scanned(&now)?;
        result.scanned_files = tracks.len() as u32;
        Ok(result)
    }

    // -- tracks ---------------------------------------------------------

    pub fn add_track(&self, track: Track) -> CoreResult<()> {
        self.db.upsert_track(&track)
    }

    pub fn get_track(&self, id: &str) -> CoreResult<Option<Track>> {
        self.db.get_track(id)
    }

    pub fn all_tracks(&self) -> CoreResult<Vec<Track>> {
        self.db.all_tracks()
    }

    pub fn search_tracks(&self, query: &str) -> CoreResult<Vec<Track>> {
        if query.trim().is_empty() {
            return self.all_tracks();
        }
        self.db.search_tracks(query.trim())
    }

    pub fn filter_tracks(&self, filter: &TrackFilter) -> CoreResult<Vec<Track>> {
        self.db.filter_tracks(filter)
    }

    // -- albums / artists ----------------------------------------------

    pub fn all_albums(&self) -> CoreResult<Vec<Album>> {
        self.db.all_albums()
    }

    pub fn all_artists(&self) -> CoreResult<Vec<Artist>> {
        self.db.all_artists()
    }

    // -- metadata -------------------------------------------------------

    pub fn metadata(&self) -> CoreResult<LibraryMetadata> {
        self.db.library_metadata()
    }

    // -- playlists ------------------------------------------------------

    pub fn create_playlist(&self, name: &str) -> CoreResult<Playlist> {
        self.db.create_playlist(name)
    }

    pub fn rename_playlist(&self, id: &str, new_name: &str) -> CoreResult<()> {
        self.db.rename_playlist(id, new_name)
    }

    pub fn delete_playlist(&self, id: &str) -> CoreResult<()> {
        self.db.delete_playlist(id)
    }

    pub fn all_playlists(&self) -> CoreResult<Vec<Playlist>> {
        self.db.all_playlists()
    }

    pub fn get_playlist(&self, id: &str) -> CoreResult<PlaylistWithTracks> {
        self.db.get_playlist(id)
    }

    pub fn add_track_to_playlist(&self, playlist_id: &str, track_id: &str) -> CoreResult<()> {
        self.db.add_track_to_playlist(playlist_id, track_id)
    }

    pub fn remove_track_from_playlist(
        &self,
        playlist_id: &str,
        track_id: &str,
    ) -> CoreResult<()> {
        self.db.remove_track_from_playlist(playlist_id, track_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Track;

    fn sample_track(id: &str, title: &str, artist: &str, album: Option<&str>) -> Track {
        Track {
            id: id.into(),
            title: title.into(),
            artist: artist.into(),
            album: album.map(String::from),
            genre: Some("Rock".into()),
            path: format!("/tmp/{id}.mp3"),
            duration_secs: 180,
            track_number: Some(1),
            year: Some(2020),
            file_format: Some("mp3".into()),
        }
    }

    #[test]
    fn add_and_retrieve_track() {
        let lib = LibraryManager::open_memory().unwrap();
        lib.add_track(sample_track("a", "Song A", "Artist X", Some("Album 1")))
            .unwrap();
        let got = lib.get_track("a").unwrap().unwrap();
        assert_eq!(got.title, "Song A");
        assert_eq!(got.artist, "Artist X");
        assert_eq!(got.album.as_deref(), Some("Album 1"));
    }

    #[test]
    fn search_matches_title_artist_album_genre() {
        let lib = LibraryManager::open_memory().unwrap();
        lib.add_track(sample_track("a", "Alpha", "Betles", Some("Abby Road")))
            .unwrap();
        lib.add_track(sample_track("b", "Beta", "Quartet", Some("Road Trip")))
            .unwrap();
        lib.add_track(sample_track("c", "Gamma", "Solo", Some("Other")))
            .unwrap();

        let road_album = lib
            .search_tracks("Road")
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect::<Vec<_>>();
        assert_eq!(road_album.len(), 2);
    }

    #[test]
    fn filter_by_artist_and_duration() {
        let lib = LibraryManager::open_memory().unwrap();
        lib.add_track(sample_track("a", "A1", "Tegan", None))
            .unwrap();
        let mut long = sample_track("b", "B1", "Tegan", None);
        long.duration_secs = 600;
        lib.add_track(long).unwrap();
        lib.add_track(sample_track("c", "C1", "Other", None))
            .unwrap();

        let f = TrackFilter {
            artist: Some("Tegan".into()),
            min_duration_secs: Some(300),
            ..Default::default()
        };
        let res = lib.filter_tracks(&f).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, "b");
    }

    #[test]
    fn library_metadata_aggregates() {
        let lib = LibraryManager::open_memory().unwrap();
        lib.add_track(sample_track("a", "A", "X", Some("Al1")))
            .unwrap();
        lib.add_track(sample_track("b", "B", "Y", Some("Al2")))
            .unwrap();
        let m = lib.metadata().unwrap();
        assert_eq!(m.total_tracks, 2);
        assert_eq!(m.total_artists, 2);
        assert_eq!(m.total_albums, 2);
        assert_eq!(m.total_duration_secs, 360);
    }

    #[test]
    fn playlist_lifecycle() {
        let lib = LibraryManager::open_memory().unwrap();
        lib.add_track(sample_track("t1", "Song", "Art", None))
            .unwrap();
        lib.add_track(sample_track("t2", "Other", "Art", None))
            .unwrap();

        let pl = lib.create_playlist("My Mix").unwrap();
        assert_eq!(pl.name, "My Mix");
        assert_eq!(pl.track_count, 0);

        lib.add_track_to_playlist(&pl.id, "t1").unwrap();
        lib.add_track_to_playlist(&pl.id, "t2").unwrap();

        let with_tracks = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(with_tracks.tracks.len(), 2);
        assert_eq!(with_tracks.playlist.track_count, 2);

        lib.add_track_to_playlist(&pl.id, "t1").unwrap();
        let with_tracks2 = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(with_tracks2.tracks.len(), 2);

        lib.remove_track_from_playlist(&pl.id, "t1").unwrap();
        let with_tracks3 = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(with_tracks3.tracks.len(), 1);
        assert_eq!(with_tracks3.tracks[0].id, "t2");

        lib.rename_playlist(&pl.id, "Renamed").unwrap();
        let with_tracks4 = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(with_tracks4.playlist.name, "Renamed");

        lib.delete_playlist(&pl.id).unwrap();
        assert!(lib.get_playlist(&pl.id).is_err());
    }

    #[test]
    fn album_and_artist_aggregation() {
        let lib = LibraryManager::open_memory().unwrap();
        lib.add_track(sample_track("a", "A1", "Duo", Some("Same Album")))
            .unwrap();
        lib.add_track(sample_track("b", "A2", "Duo", Some("Same Album")))
            .unwrap();
        lib.add_track(sample_track("c", "C1", "Solo", None))
            .unwrap();

        let albums = lib.all_albums().unwrap();
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0].track_count, 2);

        let artists = lib.all_artists().unwrap();
        assert_eq!(artists.len(), 2);
        let duo = artists.iter().find(|a| a.name == "Duo").unwrap();
        assert_eq!(duo.track_count, 2);
        assert_eq!(duo.album_count, 1);
    }
}
