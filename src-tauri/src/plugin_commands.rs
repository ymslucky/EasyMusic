//! Tauri commands for plugin management.
//!
//! These commands form the bridge between the frontend plugin management UI
//! and the Rust-side `PluginManager`. The frontend calls them to list
//! plugins, toggle enable/disable, read plugin entry-point source, and
//! install from a path.
//!
//! All commands use `State<RwLock<PluginManager>>` so both reads and writes
//! are supported through a single managed state.

use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use easy_music_core::plugins::{PluginManager, PluginStatus};

/// DTO representing a plugin for the frontend list view.
#[derive(Serialize, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub permissions: Vec<String>,
    pub hooks: Vec<String>,
}

impl From<&easy_music_core::plugins::RegisteredPlugin> for PluginInfo {
    fn from(p: &easy_music_core::plugins::RegisteredPlugin) -> Self {
        PluginInfo {
            id: p.manifest.id.clone(),
            name: p.manifest.name.clone(),
            version: p.manifest.version.clone(),
            author: p.manifest.author.clone(),
            description: p.manifest.description.clone(),
            status: match p.status {
                PluginStatus::Enabled => "enabled",
                PluginStatus::Disabled => "disabled",
                PluginStatus::Error => "error",
            }
            .to_string(),
            error: p.error.clone(),
            permissions: p
                .manifest
                .permissions
                .iter()
                .map(|p| p.to_string())
                .collect(),
            hooks: p.manifest.hooks.iter().map(|h| h.to_string()).collect(),
        }
    }
}

/// List all registered plugins.
#[tauri::command]
pub fn list_plugins(
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<Vec<PluginInfo>, String> {
    let mgr = manager.read().map_err(|e| e.to_string())?;
    Ok(mgr.all().iter().map(|p| PluginInfo::from(*p)).collect())
}

/// List only enabled plugins (used by the runtime to know what to load).
#[tauri::command]
pub fn list_enabled_plugins(
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<Vec<PluginInfo>, String> {
    let mgr = manager.read().map_err(|e| e.to_string())?;
    Ok(mgr.enabled().iter().map(|p| PluginInfo::from(*p)).collect())
}

/// Get the entry-point source code for a plugin.
///
/// The frontend runtime uses this to inject the plugin's JS into the WebView.
#[tauri::command]
pub fn get_plugin_source(
    id: String,
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<String, String> {
    let mgr = manager.read().map_err(|e| e.to_string())?;
    let plugin = mgr
        .get(&id)
        .ok_or_else(|| format!("plugin '{id}' not found"))?;
    plugin.read_entry_source().map_err(|e| e.to_string())
}

/// Enable a plugin.
#[tauri::command]
pub fn enable_plugin(
    id: String,
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<(), String> {
    let mut mgr = manager.write().map_err(|e| e.to_string())?;
    mgr.enable(&id).map_err(|e| e.to_string())
}

/// Disable a plugin.
#[tauri::command]
pub fn disable_plugin(
    id: String,
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<(), String> {
    let mut mgr = manager.write().map_err(|e| e.to_string())?;
    mgr.disable(&id).map_err(|e| e.to_string())
}

/// Get detailed info about a single plugin.
#[tauri::command]
pub fn get_plugin_info(
    id: String,
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<PluginInfo, String> {
    let mgr = manager.read().map_err(|e| e.to_string())?;
    let plugin = mgr
        .get(&id)
        .ok_or_else(|| format!("plugin '{id}' not found"))?;
    Ok(PluginInfo::from(plugin))
}

/// Reload all plugins from the plugins directory.
#[tauri::command]
pub fn reload_plugins(manager: State<'_, std::sync::RwLock<PluginManager>>) -> Result<(), String> {
    let dir = {
        let mgr = manager.read().map_err(|e| e.to_string())?;
        mgr.plugins_dir().to_path_buf()
    };
    let mut new_mgr = PluginManager::new(dir);
    new_mgr.load_all().map_err(|e| e.to_string())?;
    let mut mgr = manager.write().map_err(|e| e.to_string())?;
    *mgr = new_mgr;
    Ok(())
}

/// Install a plugin from a directory path.
///
/// Returns the freshly-registered `PluginInfo` so the frontend can append it
/// to its list without a second round-trip.
#[tauri::command]
pub fn install_plugin(
    path: String,
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<PluginInfo, String> {
    let mut mgr = manager.write().map_err(|e| e.to_string())?;
    let id = mgr
        .install_from_path(&PathBuf::from(path))
        .map_err(|e| e.to_string())?;
    let plugin = mgr
        .get(&id)
        .ok_or_else(|| format!("plugin '{id}' installed but not registered"))?;
    Ok(PluginInfo::from(plugin))
}

/// Uninstall a plugin by id.
#[tauri::command]
pub fn uninstall_plugin(
    id: String,
    manager: State<'_, std::sync::RwLock<PluginManager>>,
) -> Result<(), String> {
    let mut mgr = manager.write().map_err(|e| e.to_string())?;
    mgr.uninstall(&id).map_err(|e| e.to_string())
}
