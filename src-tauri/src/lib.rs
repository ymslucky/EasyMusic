mod commands;
mod plugin_commands;

use commands::AppState;
use std::sync::RwLock;
use tauri::Manager;

/// Tauri entry point. Registers commands and boots the app.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            // Plugin manager state: plugins live in <app_data_dir>/plugins so
            // the set survives restarts. The directory is created on first
            // launch so `load_all` always has a valid target.
            let plugins_dir = app.path().app_data_dir()?.join("plugins");
            std::fs::create_dir_all(&plugins_dir).ok();
            let mut manager = easy_music_core::plugins::PluginManager::new(plugins_dir);
            let _ = manager.load_all();
            app.manage(RwLock::new(manager));
            Ok(())
        })
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
            // plugins
            plugin_commands::list_plugins,
            plugin_commands::list_enabled_plugins,
            plugin_commands::get_plugin_source,
            plugin_commands::enable_plugin,
            plugin_commands::disable_plugin,
            plugin_commands::get_plugin_info,
            plugin_commands::reload_plugins,
            plugin_commands::install_plugin,
            plugin_commands::uninstall_plugin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
