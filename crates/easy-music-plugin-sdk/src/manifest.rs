//! Plugin manifest types — the `plugin.json` schema deserialized to Rust.
//!
//! A `PluginManifest` is the single source of truth for a plugin's identity,
//! capabilities, and declared hooks. The host validates it before the plugin
//! is ever loaded.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::hook::PluginHook;
use crate::permission::Permission;

/// Scripting/runtime engine a plugin requires.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PluginEngine {
    /// JavaScript / TypeScript entry loaded in the WebView (default).
    #[default]
    Js,
    /// WebAssembly component (reserved for future support).
    Wasm,
}

/// Parsed `plugin.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Globally unique plugin id, reverse-DNS: `com.example.lyrics`.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Semantic version string (`MAJOR.MINOR.PATCH`).
    pub version: String,
    /// Author or organization.
    pub author: String,
    /// Optional one-line description.
    #[serde(default)]
    pub description: Option<String>,
    /// Runtime engine. Defaults to `"js"`.
    #[serde(default)]
    pub engine: PluginEngine,
    /// Entry-point file relative to the plugin directory.
    pub entry: String,
    /// Permissions the plugin requests. Unknown permissions are rejected.
    #[serde(default)]
    pub permissions: Vec<Permission>,
    /// Lifecycle/event hooks the plugin subscribes to.
    #[serde(default)]
    pub hooks: Vec<PluginHook>,
    /// Minimum EasyMusic app version required (semver string).
    #[serde(default)]
    pub min_app_version: Option<String>,
}

impl PluginManifest {
    /// Parse a `plugin.json` file from its raw bytes.
    pub fn from_json_str(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Read and parse the manifest from a plugin directory's `plugin.json`.
    ///
    /// `plugin_dir` is the root directory of the plugin.
    pub fn from_plugin_dir(plugin_dir: &std::path::Path) -> Result<Self, ManifestLoadError> {
        let manifest_path = plugin_dir.join("plugin.json");
        let content = std::fs::read_to_string(&manifest_path).map_err(|e| ManifestLoadError::Io {
            path: manifest_path.clone(),
            source: e,
        })?;
        let manifest: Self = serde_json::from_str(&content).map_err(|e| {
            ManifestLoadError::Parse {
                path: manifest_path.clone(),
                source: e,
            }
        })?;
        Ok(manifest)
    }

    /// Resolve the absolute path to the plugin entry point.
    pub fn entry_path(&self, plugin_dir: &std::path::Path) -> PathBuf {
        plugin_dir.join(&self.entry)
    }

    /// Validate internal consistency of the manifest.
    ///
    /// Returns a list of validation errors (empty = valid).
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.id.is_empty() {
            errors.push("id must not be empty".into());
        }
        if !self
            .id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
        {
            errors.push(format!(
                "id '{}' contains invalid characters (allowed: a-z A-Z 0-9 . - _)",
                self.id
            ));
        }
        if self.name.is_empty() {
            errors.push("name must not be empty".into());
        }
        if self.version.is_empty() {
            errors.push("version must not be empty".into());
        }
        if self.author.is_empty() {
            errors.push("author must not be empty".into());
        }
        if self.entry.is_empty() {
            errors.push("entry must not be empty".into());
        }

        // hooks: de-duplicate to catch typos
        let mut seen = std::collections::HashSet::new();
        for h in &self.hooks {
            if !seen.insert(h) {
                errors.push(format!("duplicate hook: {h}"));
            }
        }

        // permissions: de-duplicate
        let mut seen = std::collections::HashSet::new();
        for p in &self.permissions {
            if !seen.insert(p) {
                errors.push(format!("duplicate permission: {p}"));
            }
        }

        errors
    }
}

/// Errors that can occur while loading a manifest.
#[derive(Debug)]
pub enum ManifestLoadError {
    /// Filesystem error reading `plugin.json`.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// JSON deserialization failed.
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl std::fmt::Display for ManifestLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestLoadError::Io { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            ManifestLoadError::Parse { path, source } => {
                write!(f, "failed to parse {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for ManifestLoadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_manifest() {
        let json = r#"{
            "id": "com.example.lyrics",
            "name": "Lyrics Display",
            "version": "1.0.0",
            "author": "Jane Doe",
            "description": "Synced lyrics",
            "entry": "index.js",
            "permissions": ["library:read", "network:fetch"],
            "hooks": ["onTrackChanged", "customUIPanel"]
        }"#;
        let m = PluginManifest::from_json_str(json).unwrap();
        assert_eq!(m.id, "com.example.lyrics");
        assert_eq!(m.name, "Lyrics Display");
        assert_eq!(m.entry, "index.js");
        assert_eq!(m.permissions.len(), 2);
        assert_eq!(m.hooks.len(), 2);
        assert!(m.validate().is_empty(), "valid manifest should have no errors");
    }

    #[test]
    fn parse_manifest_with_defaults() {
        let json = r#"{
            "id": "com.test.minimal",
            "name": "Minimal",
            "version": "0.1.0",
            "author": "Tester",
            "entry": "main.js"
        }"#;
        let m = PluginManifest::from_json_str(json).unwrap();
        assert!(m.permissions.is_empty());
        assert!(m.hooks.is_empty());
        assert_eq!(m.engine, PluginEngine::Js);
        assert!(m.description.is_none());
    }

    #[test]
    fn reject_manifest_with_empty_id() {
        let json = r#"{
            "id": "",
            "name": "Bad",
            "version": "1.0.0",
            "author": "X",
            "entry": "index.js"
        }"#;
        let m = PluginManifest::from_json_str(json).unwrap();
        let errors = m.validate();
        assert!(errors.iter().any(|e| e.contains("id must not be empty")));
    }

    #[test]
    fn reject_manifest_with_bad_id_chars() {
        let json = r#"{
            "id": "bad id with spaces",
            "name": "Bad",
            "version": "1.0.0",
            "author": "X",
            "entry": "index.js"
        }"#;
        let m = PluginManifest::from_json_str(json).unwrap();
        let errors = m.validate();
        assert!(errors.iter().any(|e| e.contains("invalid characters")));
    }

    #[test]
    fn detect_duplicate_hooks() {
        let json = r#"{
            "id": "com.test.dup",
            "name": "Dup",
            "version": "1.0.0",
            "author": "X",
            "entry": "index.js",
            "hooks": ["onTrackChanged", "onTrackChanged"]
        }"#;
        let m = PluginManifest::from_json_str(json).unwrap();
        let errors = m.validate();
        assert!(errors.iter().any(|e| e.contains("duplicate hook")));
    }
}
