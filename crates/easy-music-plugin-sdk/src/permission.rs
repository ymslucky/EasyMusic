//! Permission model for EasyMusic plugins.
//!
//! Plugins must declare every capability they need. The host validates the
//! declaration and the runtime enforces it before granting API access.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Capability a plugin may request in its manifest.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Read track metadata from the library.
    LibraryRead,
    /// Control playback (play/pause/skip/seek).
    PlaybackControl,
    /// Make outbound network requests (proxied by the host).
    NetworkFetch,
    /// Register a custom UI panel.
    UiPanel,
    /// Apply real-time audio transforms via Web Audio API.
    AudioTransform,
    /// Access the playlist (read/modify queue).
    PlaylistAccess,
}

impl Permission {
    /// All permissions known to the system. Used to reject unknown manifest values.
    pub fn all() -> &'static [Permission] {
        &[
            Permission::LibraryRead,
            Permission::PlaybackControl,
            Permission::NetworkFetch,
            Permission::UiPanel,
            Permission::AudioTransform,
            Permission::PlaylistAccess,
        ]
    }

    /// Check whether a permission string is recognized.
    pub fn is_valid(s: &str) -> bool {
        s.parse::<Self>().is_ok()
    }
}

impl std::str::FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "library:read" => Ok(Permission::LibraryRead),
            "playback:control" => Ok(Permission::PlaybackControl),
            "network:fetch" => Ok(Permission::NetworkFetch),
            "ui:panel" => Ok(Permission::UiPanel),
            "audio:transform" => Ok(Permission::AudioTransform),
            "playlist:access" => Ok(Permission::PlaylistAccess),
            other => Err(format!("unknown permission: {other}")),
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Permission::LibraryRead => "library:read",
            Permission::PlaybackControl => "playback:control",
            Permission::NetworkFetch => "network:fetch",
            Permission::UiPanel => "ui:panel",
            Permission::AudioTransform => "audio:transform",
            Permission::PlaylistAccess => "playlist:access",
        };
        f.write_str(s)
    }
}

/// Custom serde deserializer that validates permission strings and rejects
/// unknown values at parse time.
impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<Permission>().map_err(serde::de::Error::custom)
    }
}

// The Serialize derive above handles serialization; we only need a custom
// Deserialize to reject unknown permission strings at parse time.
// Re-impl Serialize via the derive in the struct definition is not possible
// alongside a custom Deserialize, so we implement Serialize manually.

impl Serialize for Permission {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_permissions_parse() {
        assert_eq!(
            "library:read".parse::<Permission>().unwrap(),
            Permission::LibraryRead
        );
        assert_eq!(
            "network:fetch".parse::<Permission>().unwrap(),
            Permission::NetworkFetch
        );
    }

    #[test]
    fn unknown_permission_rejected() {
        assert!("root:system".parse::<Permission>().is_err());
    }

    #[test]
    fn roundtrip_serde() {
        let p = Permission::UiPanel;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "\"ui:panel\"");
        let back: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn unknown_permission_in_manifest_rejected() {
        let json = r#"{
            "id": "com.test.bad",
            "name": "Bad",
            "version": "1.0.0",
            "author": "X",
            "entry": "index.js",
            "permissions": ["totally:made:up"]
        }"#;
        let result = crate::manifest::PluginManifest::from_json_str(json);
        assert!(result.is_err());
    }
}
