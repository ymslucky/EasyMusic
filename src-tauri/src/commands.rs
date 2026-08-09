//! Tauri command layer. Thin wrappers over `easy-music-core` — no business
//! logic lives here.

use serde::Serialize;

use easy_music_core::library::LibraryManager;

/// Demo command proving the JS → Tauri → shared-core stack is wired.
#[tauri::command]
pub fn greet(name: String) -> String {
    easy_music_core::greeting::greet(&name)
}

/// Snapshot of the library, served to the frontend.
#[derive(Serialize)]
pub struct LibraryStats {
    pub track_count: usize,
}

/// Placeholder command backed by the core `LibraryManager`.
#[tauri::command]
pub fn library_stats(library: tauri::State<'_, LibraryManager>) -> LibraryStats {
    LibraryStats {
        track_count: library.len(),
    }
}
