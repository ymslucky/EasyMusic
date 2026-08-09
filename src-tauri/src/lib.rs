mod commands;

use easy_music_core::library::LibraryManager;

/// Tauri entry point. Registers commands and boots the app.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(LibraryManager::new())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::library_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
