//! Plugin hooks / events catalog.
//!
//! Each hook is a named lifecycle or event point the host fires. Plugins
//! declare which hooks they subscribe to in their manifest; the runtime
//! dispatches only the relevant events.

use std::fmt;

use serde::{Deserialize, Serialize};

/// All hook/event names a plugin can subscribe to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PluginHook {
    /// Fired once after the plugin is loaded and registered.
    OnPluginLoaded,
    /// Fired when the current track changes.
    OnTrackChanged,
    /// Fired on play/pause/stop transitions.
    OnPlaybackStateChanged,
    /// Fired when a library scan completes.
    OnLibraryScanned,
    /// Plugin provides a custom UI panel (rendered on request).
    CustomUiPanel,
    /// Real-time audio sample processing.
    AudioTransform,
}

impl std::str::FromStr for PluginHook {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "onPluginLoaded" => Ok(PluginHook::OnPluginLoaded),
            "onTrackChanged" => Ok(PluginHook::OnTrackChanged),
            "onPlaybackStateChanged" => Ok(PluginHook::OnPlaybackStateChanged),
            "onLibraryScanned" => Ok(PluginHook::OnLibraryScanned),
            "customUIPanel" => Ok(PluginHook::CustomUiPanel),
            "audioTransform" => Ok(PluginHook::AudioTransform),
            other => Err(format!("unknown hook: {other}")),
        }
    }
}

impl fmt::Display for PluginHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PluginHook::OnPluginLoaded => "onPluginLoaded",
            PluginHook::OnTrackChanged => "onTrackChanged",
            PluginHook::OnPlaybackStateChanged => "onPlaybackStateChanged",
            PluginHook::OnLibraryScanned => "onLibraryScanned",
            PluginHook::CustomUiPanel => "customUIPanel",
            PluginHook::AudioTransform => "audioTransform",
        };
        f.write_str(s)
    }
}

// --- Custom serde to validate unknown hook names at parse time ---

impl<'de> Deserialize<'de> for PluginHook {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<PluginHook>().map_err(serde::de::Error::custom)
    }
}

impl Serialize for PluginHook {
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
    fn parse_known_hooks() {
        assert_eq!(
            "onTrackChanged".parse::<PluginHook>().unwrap(),
            PluginHook::OnTrackChanged
        );
        assert_eq!(
            "customUIPanel".parse::<PluginHook>().unwrap(),
            PluginHook::CustomUiPanel
        );
    }

    #[test]
    fn reject_unknown_hook() {
        assert!("onCatastrophe".parse::<PluginHook>().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let h = PluginHook::OnLibraryScanned;
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(json, "\"onLibraryScanned\"");
        let back: PluginHook = serde_json::from_str(&json).unwrap();
        assert_eq!(back, h);
    }
}
