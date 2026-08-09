//! Playback engine core: transport state and control surface.
//!
//! The actual audio backend (rodio/symphonia/CPAL) plugs in behind this
//! interface; the state machine itself is backend-agnostic and testable.

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use crate::library::Track;

/// Current transport state of the playback engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Backend-agnostic playback engine state machine.
#[derive(Debug)]
pub struct PlaybackEngine {
    state: PlaybackState,
    current: Option<Track>,
    position_secs: u32,
}

impl Default for PlaybackEngine {
    fn default() -> Self {
        Self {
            state: PlaybackState::Stopped,
            current: None,
            position_secs: 0,
        }
    }
}

impl PlaybackEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> PlaybackState {
        self.state
    }

    pub fn current(&self) -> Option<&Track> {
        self.current.as_ref()
    }

    pub fn position_secs(&self) -> u32 {
        self.position_secs
    }

    /// Load a track and start playback.
    pub fn play(&mut self, track: Track) -> CoreResult<()> {
        if track.path.is_empty() {
            return Err(CoreError::Playback("track has no path".into()));
        }
        self.current = Some(track);
        self.state = PlaybackState::Playing;
        self.position_secs = 0;
        Ok(())
    }

    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    pub fn resume(&mut self) -> CoreResult<()> {
        if self.state == PlaybackState::Paused {
            self.state = PlaybackState::Playing;
            Ok(())
        } else {
            Err(CoreError::Playback(
                "cannot resume: engine is not paused".into(),
            ))
        }
    }

    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.position_secs = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track() -> Track {
        Track {
            id: "t1".into(),
            title: "Test".into(),
            artist: "Tester".into(),
            album: None,
            path: "/tmp/test.mp3".into(),
            duration_secs: 120,
        }
    }

    #[test]
    fn play_pause_resume_stop_cycle() {
        let mut engine = PlaybackEngine::new();
        assert_eq!(engine.state(), PlaybackState::Stopped);

        engine.play(sample_track()).unwrap();
        assert_eq!(engine.state(), PlaybackState::Playing);
        assert_eq!(engine.current().unwrap().id, "t1");

        engine.pause();
        assert_eq!(engine.state(), PlaybackState::Paused);
        engine.resume().unwrap();
        assert_eq!(engine.state(), PlaybackState::Playing);

        engine.stop();
        assert_eq!(engine.state(), PlaybackState::Stopped);
    }

    #[test]
    fn play_rejects_track_without_path() {
        let mut engine = PlaybackEngine::new();
        let mut track = sample_track();
        track.path.clear();
        assert!(engine.play(track).is_err());
    }
}
