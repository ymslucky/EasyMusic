mod commands;

use commands::AppState;

/// Tauri entry point. Registers commands and boots the app.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // greeting / demo
            commands::greet,
            // library lifecycle
            commands::library_open_db,
            commands::library_scan,
            commands::library_metadata,
            // tracks
            commands::tracks_all,
            commands::track_get,
            commands::tracks_search,
            commands::tracks_filter,
            // albums / artists
            commands::albums_all,
            commands::artists_all,
            // playlists
            commands::playlist_create,
            commands::playlist_rename,
            commands::playlist_delete,
            commands::playlists_all,
            commands::playlist_get,
            commands::playlist_add_track,
            commands::playlist_remove_track,
            // playback
            commands::playback_status,
            commands::playback_play_track,
            commands::playback_play_queue,
            commands::playback_pause,
            commands::playback_resume,
            commands::playback_stop,
            commands::playback_seek,
            commands::playback_set_volume,
            commands::playback_next,
            commands::playback_previous,
            commands::playback_set_repeat,
            commands::playback_toggle_shuffle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
