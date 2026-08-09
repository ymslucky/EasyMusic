//! Plugin host and loader — discovers, validates, and manages plugins.
//!
//! The `PluginManager` scans a plugin directory, parses each plugin's
//! `plugin.json` manifest, validates it, tracks enabled/disabled state, and
//! exposes metadata to the frontend. The actual JS code execution happens in
//! the WebView runtime; this module handles the Rust-side lifecycle.
//!
//! ## Structure
//!
//! Each subdirectory of the plugins directory that contains a `plugin.json`
//! is treated as one plugin. Example layout:
//!
//! ```text
//! plugins/
//! └── lyrics-display/
//!     ├── plugin.json      ← manifest
//!     ├── index.js          ← entry point (JS)
//!     └── README.md
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use easy_music_plugin_sdk::manifest::{ManifestLoadError, PluginManifest};

// ── Errors ──────────────────────────────────────────────────────────────

/// Errors produced during plugin discovery, loading, or management.
#[derive(Debug)]
pub enum PluginError {
    /// Failed to read or parse a manifest.
    Manifest(ManifestLoadError),
    /// Manifest parsed but failed validation.
    Validation { plugin_id: String, errors: Vec<String> },
    /// A plugin directory path was not found.
    DirNotFound(PathBuf),
    /// A plugin with this id is already registered.
    DuplicateId(String),
    /// Entry-point file does not exist on disk.
    EntryNotFound { plugin_id: String, path: PathBuf },
    /// Plugin not found in the manager.
    NotRegistered(String),
    /// Generic I/O error.
    Io(std::io::Error),
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginError::Manifest(e) => write!(f, "manifest error: {e}"),
            PluginError::Validation { plugin_id, errors } => {
                write!(f, "manifest for '{plugin_id}' is invalid: {}", errors.join("; "))
            }
            PluginError::DirNotFound(p) => write!(f, "plugins directory not found: {}", p.display()),
            PluginError::DuplicateId(id) => write!(f, "plugin id '{id}' is already registered"),
            PluginError::EntryNotFound { plugin_id, path } => {
                write!(f, "plugin '{plugin_id}' entry point not found: {}", path.display())
            }
            PluginError::NotRegistered(id) => write!(f, "plugin '{id}' is not registered"),
            PluginError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for PluginError {}

impl From<ManifestLoadError> for PluginError {
    fn from(e: ManifestLoadError) -> Self {
        PluginError::Manifest(e)
    }
}

impl From<std::io::Error> for PluginError {
    fn from(e: std::io::Error) -> Self {
        PluginError::Io(e)
    }
}

// Re-export for convenience.
pub type PluginResult<T> = Result<T, PluginError>;

// ── Plugin state ────────────────────────────────────────────────────────

/// Runtime status of a registered plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginStatus {
    /// Plugin loaded but disabled by the user.
    Disabled,
    /// Plugin loaded and enabled.
    Enabled,
    /// Plugin failed to load (see `error` field on `RegisteredPlugin`).
    Error,
}

impl Default for PluginStatus {
    fn default() -> Self {
        PluginStatus::Enabled
    }
}

/// A plugin that has been discovered and loaded by the manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredPlugin {
    /// Parsed manifest.
    pub manifest: PluginManifest,
    /// Absolute path to the plugin directory.
    pub dir: PathBuf,
    /// Current status.
    pub status: PluginStatus,
    /// Error message if status is `Error`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl RegisteredPlugin {
    /// Read the entry-point source as a UTF-8 string.
    ///
    /// Used by the Tauri command layer to serve the plugin JS to the
    /// frontend runtime.
    pub fn read_entry_source(&self) -> PluginResult<String> {
        let entry_path = self.manifest.entry_path(&self.dir);
        if !entry_path.exists() {
            return Err(PluginError::EntryNotFound {
                plugin_id: self.manifest.id.clone(),
                path: entry_path,
            });
        }
        Ok(std::fs::read_to_string(&entry_path)?)
    }

    /// Check if the plugin subscribes to a given hook name.
    pub fn has_hook(&self, hook_name: &str) -> bool {
        self.manifest.hooks.iter().any(|h| h.to_string() == hook_name)
    }
}

// ── Plugin Manager ──────────────────────────────────────────────────────

/// Central plugin manager. Scans a directory, loads manifests, and tracks
/// state.
#[derive(Debug)]
pub struct PluginManager {
    /// The root plugins directory.
    plugins_dir: PathBuf,
    /// Registered plugins keyed by id.
    plugins: HashMap<String, RegisteredPlugin>,
}

impl PluginManager {
    /// Create a new manager pointing at the given plugins directory.
    /// Does NOT scan — call `load_all()` to discover plugins.
    pub fn new<P: Into<PathBuf>>(plugins_dir: P) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            plugins: HashMap::new(),
        }
    }

    /// Get the plugins directory path.
    pub fn plugins_dir(&self) -> &Path {
        &self.plugins_dir
    }

    /// Scan the plugins directory and load all valid plugins.
    ///
    /// Each subdirectory containing a `plugin.json` is treated as a plugin.
    /// Invalid plugins are recorded with status `Error` rather than
    /// aborting the entire scan.
    pub fn load_all(&mut self) -> PluginResult<()> {
        if !self.plugins_dir.exists() {
            return Err(PluginError::DirNotFound(self.plugins_dir.clone()));
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                // Skip directories without a manifest
                continue;
            }

            let plugin_id_for_error = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());

            match self.load_single(&path) {
                Ok(rp) => {
                    self.plugins.insert(rp.manifest.id.clone(), rp);
                }
                Err(e) => {
                    // Record as an error plugin if possible, or just log
                    eprintln!("[plugins] failed to load from {}: {e}", path.display());
                    // Try to record an error entry so the UI can show it
                    let error_plugin = RegisteredPlugin {
                        manifest: PluginManifest {
                            id: plugin_id_for_error.clone(),
                            name: plugin_id_for_error.clone(),
                            version: "unknown".into(),
                            author: "unknown".into(),
                            description: None,
                            engine: Default::default(),
                            entry: String::new(),
                            permissions: vec![],
                            hooks: vec![],
                            min_app_version: None,
                        },
                        dir: path,
                        status: PluginStatus::Error,
                        error: Some(e.to_string()),
                    };
                    self.plugins.insert(plugin_id_for_error, error_plugin);
                }
            }
        }

        Ok(())
    }

    /// Load a single plugin from its directory.
    fn load_single(&self, plugin_dir: &Path) -> PluginResult<RegisteredPlugin> {
        let manifest = PluginManifest::from_plugin_dir(plugin_dir)?;

        // Validate
        let errors = manifest.validate();
        if !errors.is_empty() {
            return Err(PluginError::Validation {
                plugin_id: manifest.id.clone(),
                errors,
            });
        }

        // Check entry point exists
        let entry_path = manifest.entry_path(plugin_dir);
        if !entry_path.exists() {
            return Err(PluginError::EntryNotFound {
                plugin_id: manifest.id.clone(),
                path: entry_path,
            });
        }

        // Check for duplicate
        if self.plugins.contains_key(&manifest.id) {
            return Err(PluginError::DuplicateId(manifest.id));
        }

        Ok(RegisteredPlugin {
            manifest,
            dir: plugin_dir.to_path_buf(),
            status: PluginStatus::Enabled,
            error: None,
        })
    }

    /// Load a single plugin from a directory path and register it.
    ///
    /// This is the programmatic equivalent of `load_all()` for one plugin.
    /// Useful for the "install from path" UI feature.
    pub fn install_from_path(&mut self, plugin_dir: &Path) -> PluginResult<String> {
        let rp = self.load_single(plugin_dir)?;
        let id = rp.manifest.id.clone();
        self.plugins.insert(id.clone(), rp);
        Ok(id)
    }

    /// Enable a registered plugin.
    pub fn enable(&mut self, id: &str) -> PluginResult<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotRegistered(id.into()))?;
        if plugin.status == PluginStatus::Error {
            return Err(PluginError::NotRegistered(format!(
                "cannot enable plugin '{id}' — it failed to load"
            )));
        }
        plugin.status = PluginStatus::Enabled;
        Ok(())
    }

    /// Disable a registered plugin.
    pub fn disable(&mut self, id: &str) -> PluginResult<()> {
        let plugin = self
            .plugins
            .get_mut(id)
            .ok_or_else(|| PluginError::NotRegistered(id.into()))?;
        plugin.status = PluginStatus::Disabled;
        Ok(())
    }

    /// Remove a plugin from the registry.
    pub fn uninstall(&mut self, id: &str) -> PluginResult<()> {
        self.plugins
            .remove(id)
            .ok_or_else(|| PluginError::NotRegistered(id.into()))?;
        Ok(())
    }

    /// Look up a registered plugin by id.
    pub fn get(&self, id: &str) -> Option<&RegisteredPlugin> {
        self.plugins.get(id)
    }

    /// Get all registered plugins.
    pub fn all(&self) -> Vec<&RegisteredPlugin> {
        self.plugins.values().collect()
    }

    /// Get only enabled plugins.
    pub fn enabled(&self) -> Vec<&RegisteredPlugin> {
        self.plugins
            .values()
            .filter(|p| p.status == PluginStatus::Enabled)
            .collect()
    }

    /// Number of registered plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_plugin_dir(root: &Path, id: &str, entry_content: &str) -> PathBuf {
        let dir = root.join(id);
        std::fs::create_dir_all(&dir).unwrap();

        let manifest = format!(
            r#"{{
                "id": "{id}",
                "name": "Test Plugin",
                "version": "1.0.0",
                "author": "Tester",
                "description": "A test plugin",
                "entry": "index.js",
                "permissions": ["library:read"],
                "hooks": ["onTrackChanged"]
            }}"#
        );
        std::fs::write(dir.join("plugin.json"), manifest).unwrap();

        let mut f = std::fs::File::create(dir.join("index.js")).unwrap();
        f.write_all(entry_content.as_bytes()).unwrap();

        dir
    }

    #[test]
    fn load_all_discovers_valid_plugins() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let root = tmp.into_temp_path();
        std::fs::remove_file(&root).unwrap();
        std::fs::create_dir_all(&root).unwrap();

        make_plugin_dir(&root, "com.test.one", "console.log('one');");
        make_plugin_dir(&root, "com.test.two", "console.log('two');");

        let mut mgr = PluginManager::new(&root);
        mgr.load_all().unwrap();

        assert_eq!(mgr.len(), 2);
        assert!(mgr.get("com.test.one").is_some());
        assert!(mgr.get("com.test.two").is_some());
        assert_eq!(mgr.get("com.test.one").unwrap().status, PluginStatus::Enabled);
    }

    #[test]
    fn load_all_records_error_for_invalid_manifest() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let root = tmp.into_temp_path();
        std::fs::remove_file(&root).unwrap();
        std::fs::create_dir_all(&root).unwrap();

        // Valid plugin
        make_plugin_dir(&root, "com.test.good", "// good");

        // Invalid plugin (empty id)
        let bad_dir = root.join("com.test.bad");
        std::fs::create_dir_all(&bad_dir).unwrap();
        std::fs::write(
            bad_dir.join("plugin.json"),
            r#"{"id":"","name":"Bad","version":"1.0.0","author":"X","entry":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(bad_dir.join("index.js"), "// bad").unwrap();

        let mut mgr = PluginManager::new(&root);
        mgr.load_all().unwrap();

        assert_eq!(mgr.len(), 2);
        assert_eq!(
            mgr.get("com.test.bad").unwrap().status,
            PluginStatus::Error
        );
        assert_eq!(mgr.enabled().len(), 1);
    }

    #[test]
    fn enable_disable_cycle() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let root = tmp.into_temp_path();
        std::fs::remove_file(&root).unwrap();
        std::fs::create_dir_all(&root).unwrap();

        make_plugin_dir(&root, "com.test.toggle", "// toggle");

        let mut mgr = PluginManager::new(&root);
        mgr.load_all().unwrap();

        assert_eq!(mgr.get("com.test.toggle").unwrap().status, PluginStatus::Enabled);

        mgr.disable("com.test.toggle").unwrap();
        assert_eq!(
            mgr.get("com.test.toggle").unwrap().status,
            PluginStatus::Disabled
        );

        mgr.enable("com.test.toggle").unwrap();
        assert_eq!(mgr.get("com.test.toggle").unwrap().status, PluginStatus::Enabled);
    }

    #[test]
    fn install_from_path_works() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let root = tmp.into_temp_path();
        std::fs::remove_file(&root).unwrap();
        std::fs::create_dir_all(&root).unwrap();

        let plugin_dir = make_plugin_dir(&root, "com.test.installed", "// installed");

        let plugins_root = root.join("plugins_store");
        std::fs::create_dir_all(&plugins_root).unwrap();

        let mut mgr = PluginManager::new(&plugins_root);
        let id = mgr.install_from_path(&plugin_dir).unwrap();
        assert_eq!(id, "com.test.installed");
        assert!(mgr.get("com.test.installed").is_some());
    }

    #[test]
    fn duplicate_id_rejected() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let root = tmp.into_temp_path();
        std::fs::remove_file(&root).unwrap();
        std::fs::create_dir_all(&root).unwrap();

        make_plugin_dir(&root, "com.test.dup", "// first");

        let mut mgr = PluginManager::new(&root);
        mgr.load_all().unwrap();

        // Try to install the same plugin again
        let result = mgr.install_from_path(&root.join("com.test.dup"));
        assert!(result.is_err());
    }

    #[test]
    fn read_entry_source_returns_js() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let root = tmp.into_temp_path();
        std::fs::remove_file(&root).unwrap();
        std::fs::create_dir_all(&root).unwrap();

        make_plugin_dir(&root, "com.test.src", "export function onLoad(ctx){}");

        let mut mgr = PluginManager::new(&root);
        mgr.load_all().unwrap();

        let plugin = mgr.get("com.test.src").unwrap();
        let source = plugin.read_entry_source().unwrap();
        assert!(source.contains("onLoad"));
    }

    #[test]
    fn missing_plugins_dir_returns_error() {
        let mut mgr = PluginManager::new("/nonexistent/path/that/should/not/exist");
        let result = mgr.load_all();
        assert!(result.is_err());
    }

    #[test]
    fn uninstall_removes_plugin() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let root = tmp.into_temp_path();
        std::fs::remove_file(&root).unwrap();
        std::fs::create_dir_all(&root).unwrap();

        make_plugin_dir(&root, "com.test.remove", "// remove me");

        let mut mgr = PluginManager::new(&root);
        mgr.load_all().unwrap();
        assert_eq!(mgr.len(), 1);

        mgr.uninstall("com.test.remove").unwrap();
        assert!(mgr.is_empty());
    }
}
