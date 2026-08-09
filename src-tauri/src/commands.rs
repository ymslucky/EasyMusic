//! Tauri command layer. Thin wrappers over `easy-music-core` — no business
//! logic lives here.
//!
//! The library and playback engine live behind `tauri::State` as a single
//! `AppState` struct so the frontend can reach them through `invoke()`.

use std::path::PathBuf;
use std::sync::Mutex;

use easy_music_core::models::{
    Album, Artist, LibraryMetadata, Playlist, PlaylistWithTracks, ScanResult, Track, TrackFilter,
};
use easy_music_core::playback::{NullAudioSink, PlaybackEngine, PlaybackStatus, RepeatMode};
use easy_music_core::LibraryManager;

use serde::Serialize;

/// The shared app state managed by Tauri. Each field is wrapped in a `Mutex`
/// so commands can run concurrently from the frontend's async runtime.
pub struct AppState {
    pub library: Mutex<LibraryManager>,
    pub playback: Mutex<PlaybackEngine<NullAudioSink>>,
}

impl AppState {
    /// Create the default state with an in-memory library. In production the
    /// frontend will call `library_open_db` to switch to a persistent file.
    pub fn new() -> Self {
        let library =
            LibraryManager::open_memory().expect("failed to open default in-memory library");
        let playback = PlaybackEngine::new(NullAudioSink::new());
        Self {
            library: Mutex::new(library),
            playback: Mutex::new(playback),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

/// A serializable error envelope. Tauri commands that return `Result<T, _>`
/// must use a type that implements `Serialize`.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
    pub kind: String,
}

impl From<easy_music_core::CoreError> for CommandError {
    fn from(e: easy_music_core::CoreError) -> Self {
        let kind = match &e {
            easy_music_core::CoreError::NotFound(_) => "not_found",
            easy_music_core::CoreError::Playback(_) => "playback",
            easy_music_core::CoreError::Database(_) => "database",
            easy_music_core::CoreError::Io(_) => "io",
            easy_music_core::CoreError::TagParse(_) => "tag_parse",
            easy_music_core::CoreError::Invalid(_) => "invalid",
            easy_music_core::CoreError::Internal(_) => "internal",
        };
        CommandError {
            message: e.to_string(),
            kind: kind.to_string(),
        }
    }
}

type CmdResult<T> = Result<T, CommandError>;

// ---------------------------------------------------------------------------
// Greeting (demo)
// ---------------------------------------------------------------------------

/// Demo command proving the JS → Tauri → shared-core stack is wired.
#[tauri::command]
pub fn greet(name: String) -> String {
    easy_music_core::greeting::greet(&name)
}

// ---------------------------------------------------------------------------
// Library: lifecycle
// ---------------------------------------------------------------------------

/// Reopen the library against a file-backed SQLite DB so data persists.
#[tauri::command]
pub fn library_open_db(db_path: String, state: tauri::State<'_, AppState>) -> CmdResult<()> {
    let lib = LibraryManager::open(&db_path)?;
    *state.library.lock().unwrap() = lib;
    Ok(())
}

/// Scan a directory and index all discovered audio files.
#[tauri::command]
pub fn library_scan(root: String, state: tauri::State<'_, AppState>) -> CmdResult<ScanResult> {
    let lib = state.library.lock().unwrap();
    Ok(lib.scan_directory(&PathBuf::from(&root))?)
}

/// Aggregate stats about the library.
#[tauri::command]
pub fn library_metadata(state: tauri::State<'_, AppState>) -> CmdResult<LibraryMetadata> {
    let lib = state.library.lock().unwrap();
    Ok(lib.metadata()?)
}

// ---------------------------------------------------------------------------
// Library: tracks
// ---------------------------------------------------------------------------

/// Return every track in the library, ordered by title.
#[tauri::command]
pub fn tracks_all(state: tauri::State<'_, AppState>) -> CmdResult<Vec<Track>> {
    let lib = state.library.lock().unwrap();
    Ok(lib.all_tracks()?)
}

/// Fetch a single track by id.
#[tauri::command]
pub fn track_get(id: String, state: tauri::State<'_, AppState>) -> CmdResult<Option<Track>> {
    let lib = state.library.lock().unwrap();
    Ok(lib.get_track(&id)?)
}

/// Full-text search across title, artist, album, and genre.
#[tauri::command]
pub fn tracks_search(query: String, state: tauri::State<'_, AppState>) -> CmdResult<Vec<Track>> {
    let lib = state.library.lock().unwrap();
    Ok(lib.search_tracks(&query)?)
}

/// Structured filter (artist, album, genre, duration range).
#[tauri::command]
pub fn tracks_filter(
    filter: TrackFilter,
    state: tauri::State<'_, AppState>,
) -> CmdResult<Vec<Track>> {
    let lib = state.library.lock().unwrap();
    Ok(lib.filter_tracks(&filter)?)
}

// ---------------------------------------------------------------------------
// Library: albums / artists
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn albums_all(state: tauri::State<'_, AppState>) -> CmdResult<Vec<Album>> {
    let lib = state.library.lock().unwrap();
    Ok(lib.all_albums()?)
}

#[tauri::command]
pub fn artists_all(state: tauri::State<'_, AppState>) -> CmdResult<Vec<Artist>> {
    let lib = state.library.lock().unwrap();
    Ok(lib.all_artists()?)
}

// ---------------------------------------------------------------------------
// Library: playlists (CRUD)
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn playlist_create(name: String, state: tauri::State<'_, AppState>) -> CmdResult<Playlist> {
    let lib = state.library.lock().unwrap();
    Ok(lib.create_playlist(&name)?)
}

#[tauri::command]
pub fn playlist_rename(
    id: String,
    new_name: String,
    state: tauri::State<'_, AppState>,
) -> CmdResult<()> {
    let lib = state.library.lock().unwrap();
    Ok(lib.rename_playlist(&id, &new_name)?)
}

#[tauri::command]
pub fn playlist_delete(id: String, state: tauri::State<'_, AppState>) -> CmdResult<()> {
    let lib = state.library.lock().unwrap();
    Ok(lib.delete_playlist(&id)?)
}

#[tauri::command]
pub fn playlists_all(state: tauri::State<'_, AppState>) -> CmdResult<Vec<Playlist>> {
    let lib = state.library.lock().unwrap();
    Ok(lib.all_playlists()?)
}

#[tauri::command]
pub fn playlist_get(
    id: String,
    state: tauri::State<'_, AppState>,
) -> CmdResult<PlaylistWithTracks> {
    let lib = state.library.lock().unwrap();
    Ok(lib.get_playlist(&id)?)
}

#[tauri::command]
pub fn playlist_add_track(
    playlist_id: String,
    track_id: String,
    state: tauri::State<'_, AppState>,
) -> CmdResult<()> {
    let lib = state.library.lock().unwrap();
    Ok(lib.add_track_to_playlist(&playlist_id, &track_id)?)
}

#[tauri::command]
pub fn playlist_remove_track(
    playlist_id: String,
    track_id: String,
    state: tauri::State<'_, AppState>,
) -> CmdResult<()> {
    let lib = state.library.lock().unwrap();
    Ok(lib.remove_track_from_playlist(&playlist_id, &track_id)?)
}

// ---------------------------------------------------------------------------
// Playback engine
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn playback_status(state: tauri::State<'_, AppState>) -> PlaybackStatus {
    let engine = state.playback.lock().unwrap();
    engine.status()
}

/// Play a single track (replaces the queue).
#[tauri::command]
pub fn playback_play_track(
    track: Track,
    state: tauri::State<'_, AppState>,
) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.play(track)?;
    Ok(engine.status())
}

/// Replace the queue with `tracks` and start playing the first entry.
#[tauri::command]
pub fn playback_play_queue(
    tracks: Vec<Track>,
    state: tauri::State<'_, AppState>,
) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.play_queue(tracks)?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_pause(state: tauri::State<'_, AppState>) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.pause()?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_resume(state: tauri::State<'_, AppState>) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.resume()?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_stop(state: tauri::State<'_, AppState>) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.stop()?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_seek(secs: u32, state: tauri::State<'_, AppState>) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.seek(secs)?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_set_volume(
    volume: f32,
    state: tauri::State<'_, AppState>,
) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.set_volume(volume)?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_next(state: tauri::State<'_, AppState>) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.advance()?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_previous(state: tauri::State<'_, AppState>) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.previous()?;
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_set_repeat(
    mode: RepeatMode,
    state: tauri::State<'_, AppState>,
) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.set_repeat(mode);
    Ok(engine.status())
}

#[tauri::command]
pub fn playback_toggle_shuffle(state: tauri::State<'_, AppState>) -> CmdResult<PlaybackStatus> {
    let mut engine = state.playback.lock().unwrap();
    engine.toggle_shuffle();
    Ok(engine.status())
}
