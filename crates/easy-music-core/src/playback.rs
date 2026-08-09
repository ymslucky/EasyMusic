//! Playback engine — transport state machine, queue, volume, seek, and
//! next/previous navigation.
//!
//! The actual audio output is abstracted behind the [`AudioSink`] trait.
//! In production the Tauri layer installs a `rodio`-backed sink; in tests
//! a no-op sink keeps the state machine fully exercisable without a sound
//! device.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};
use crate::models::Track;

/// Current transport state of the playback engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Repeat mode for the queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

/// Snapshot of the engine state returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStatus {
    pub state: PlaybackState,
    pub current_track: Option<Track>,
    pub position_secs: u32,
    pub duration_secs: u32,
    pub volume: f32,
    pub repeat: RepeatMode,
    pub shuffle: bool,
    pub queue_length: usize,
    pub queue_index: Option<usize>,
}

/// Abstraction over the audio output device.
///
/// Implementations are responsible for actually decoding + playing the PCM
/// samples. The engine drives them via these callbacks.
pub trait AudioSink: Send + 'static {
    /// Begin playing `track` from `start_secs`.
    fn play(&mut self, track: &Track, start_secs: u32) -> CoreResult<()>;
    /// Pause output.
    fn pause(&mut self) -> CoreResult<()>;
    /// Resume output.
    fn resume(&mut self) -> CoreResult<()>;
    /// Stop output and release the device.
    fn stop(&mut self) -> CoreResult<()>;
    /// Seek to `secs` within the current track.
    fn seek(&mut self, secs: u32) -> CoreResult<()>;
    /// Set the software volume (0.0 ..= 1.0).
    fn set_volume(&mut self, vol: f32) -> CoreResult<()>;
    /// Poll the current playback position (seconds elapsed).
    fn position_secs(&self) -> u32;
}

/// A no-op sink used for tests and for headless runs where no audio device
/// is available. It still tracks volume/position bookkeeping so the engine's
/// state machine is fully drivable.
#[derive(Debug, Default)]
pub struct NullAudioSink {
    position: u32,
    volume: f32,
}

impl NullAudioSink {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AudioSink for NullAudioSink {
    fn play(&mut self, _track: &Track, start_secs: u32) -> CoreResult<()> {
        self.position = start_secs;
        Ok(())
    }
    fn pause(&mut self) -> CoreResult<()> {
        Ok(())
    }
    fn resume(&mut self) -> CoreResult<()> {
        Ok(())
    }
    fn stop(&mut self) -> CoreResult<()> {
        self.position = 0;
        Ok(())
    }
    fn seek(&mut self, secs: u32) -> CoreResult<()> {
        self.position = secs;
        Ok(())
    }
    fn set_volume(&mut self, vol: f32) -> CoreResult<()> {
        self.volume = vol;
        Ok(())
    }
    fn position_secs(&self) -> u32 {
        self.position
    }
}

/// Backend-agnostic playback engine with a queue.
pub struct PlaybackEngine<S: AudioSink> {
    state: PlaybackState,
    queue: VecDeque<Track>,
    /// Index into `queue` of the "current" track. None when stopped/empty.
    index: Option<usize>,
    position_secs: u32,
    volume: f32,
    repeat: RepeatMode,
    shuffle: bool,
    sink: S,
}

impl<S: AudioSink> PlaybackEngine<S> {
    /// Create a new engine wrapping the given audio sink.
    pub fn new(sink: S) -> Self {
        Self {
            state: PlaybackState::Stopped,
            queue: VecDeque::new(),
            index: None,
            position_secs: 0,
            volume: 1.0,
            repeat: RepeatMode::Off,
            shuffle: false,
            sink,
        }
    }

    // -- accessors ------------------------------------------------------

    pub fn state(&self) -> PlaybackState {
        self.state
    }

    pub fn current(&self) -> Option<&Track> {
        self.index.and_then(|i| self.queue.get(i))
    }

    pub fn position_secs(&self) -> u32 {
        self.position_secs
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn repeat(&self) -> RepeatMode {
        self.repeat
    }

    pub fn shuffle(&self) -> bool {
        self.shuffle
    }

    pub fn queue(&self) -> &VecDeque<Track> {
        &self.queue
    }

    pub fn queue_index(&self) -> Option<usize> {
        self.index
    }

    /// Build a serializable status snapshot for the frontend.
    pub fn status(&self) -> PlaybackStatus {
        PlaybackStatus {
            state: self.state,
            current_track: self.current().cloned(),
            position_secs: self.position_secs,
            duration_secs: self.current().map(|t| t.duration_secs).unwrap_or(0),
            volume: self.volume,
            repeat: self.repeat,
            shuffle: self.shuffle,
            queue_length: self.queue.len(),
            queue_index: self.index,
        }
    }

    // -- queue management ----------------------------------------------

    /// Replace the queue with `tracks` and immediately start playing the
    /// first entry.
    pub fn play_queue(&mut self, tracks: Vec<Track>) -> CoreResult<()> {
        if tracks.is_empty() {
            return Err(CoreError::Invalid("cannot play an empty queue".into()));
        }
        self.queue = tracks.into_iter().collect();
        if self.shuffle {
            self.shuffle_queue();
        }
        self.index = Some(0);
        self.start_current(0)
    }

    /// Append tracks to the end of the queue without interrupting playback.
    pub fn enqueue(&mut self, tracks: Vec<Track>) {
        for t in tracks {
            self.queue.push_back(t);
        }
    }

    /// Clear the queue and stop playback.
    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.index = None;
        let _ = self.sink.stop();
        self.state = PlaybackState::Stopped;
        self.position_secs = 0;
    }

    // -- transport controls --------------------------------------------

    /// Play `track` by loading it into the sink.
    ///
    /// If the track is already in the queue, jump to it; otherwise replace
    /// the queue with this single track.
    pub fn play(&mut self, track: Track) -> CoreResult<()> {
        if track.path.is_empty() {
            return Err(CoreError::Playback("track has no path".into()));
        }
        // If this exact track (by id) already sits in the queue, jump to it.
        if let Some(pos) = self.queue.iter().position(|t| t.id == track.id) {
            self.index = Some(pos);
            return self.start_current(0);
        }
        self.queue.clear();
        self.queue.push_back(track);
        self.index = Some(0);
        self.start_current(0)
    }

    pub fn pause(&mut self) -> CoreResult<()> {
        if self.state == PlaybackState::Playing {
            self.sink.pause()?;
            self.state = PlaybackState::Paused;
        }
        Ok(())
    }

    pub fn resume(&mut self) -> CoreResult<()> {
        match self.state {
            PlaybackState::Paused => {
                self.sink.resume()?;
                self.state = PlaybackState::Playing;
                Ok(())
            }
            _ => Err(CoreError::Playback(
                "cannot resume: engine is not paused".into(),
            )),
        }
    }

    pub fn stop(&mut self) -> CoreResult<()> {
        self.sink.stop()?;
        self.state = PlaybackState::Stopped;
        self.position_secs = 0;
        self.index = None;
        Ok(())
    }

    /// Seek to `secs` within the current track.
    pub fn seek(&mut self, secs: u32) -> CoreResult<()> {
        if self.current().is_none() {
            return Err(CoreError::Playback("no track loaded to seek".into()));
        }
        self.sink.seek(secs)?;
        self.position_secs = secs;
        Ok(())
    }

    /// Set the software volume (0.0 ..= 1.0). Clamped automatically.
    pub fn set_volume(&mut self, vol: f32) -> CoreResult<()> {
        let clamped = vol.clamp(0.0, 1.0);
        self.volume = clamped;
        self.sink.set_volume(clamped)
    }

    // -- navigation -----------------------------------------------------

    /// Advance to the next track (or repeat, or stop at end).
    pub fn advance(&mut self) -> CoreResult<()> {
        let idx = self
            .index
            .ok_or_else(|| CoreError::Playback("cannot advance: queue position is none".into()))?;
        let len = self.queue.len();
        let next_idx = match self.repeat {
            RepeatMode::One => idx,
            RepeatMode::All => (idx + 1) % len,
            RepeatMode::Off => {
                if idx + 1 >= len {
                    self.stop()?;
                    return Ok(());
                }
                idx + 1
            }
        };
        self.index = Some(next_idx);
        self.start_current(0)
    }

    pub fn previous(&mut self) -> CoreResult<()> {
        let idx = self
            .index
            .ok_or_else(|| CoreError::Playback("cannot go back: queue position is none".into()))?;
        let len = self.queue.len();
        let prev_idx = match self.repeat {
            RepeatMode::One => idx,
            RepeatMode::All => {
                if idx == 0 {
                    len - 1
                } else {
                    idx - 1
                }
            }
            RepeatMode::Off => {
                if idx == 0 {
                    return self.start_current(0);
                }
                idx - 1
            }
        };
        self.index = Some(prev_idx);
        self.start_current(0)
    }

    // -- repeat / shuffle ----------------------------------------------

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        if self.shuffle {
            self.shuffle_queue();
        }
    }

    /// Shuffle the queue but keep the current track first.
    fn shuffle_queue(&mut self) {
        if self.queue.len() <= 1 {
            return;
        }
        let current_id = self.current().map(|t| t.id.clone());
        let mut items: Vec<Track> = self.queue.drain(..).collect();
        if let Some(id) = &current_id {
            items.sort_by_key(|t| t.id != *id);
        }
        let n = items.len();
        if n > 1 {
            let mut seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0xABCDEF);
            for i in (1..n).rev() {
                seed = seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let j = 1 + (seed as usize) % i;
                items.swap(i, j);
            }
        }
        self.queue = items.into_iter().collect();
        self.index = Some(0);
    }

    // -- internals ------------------------------------------------------

    /// Load the track at `self.index` into the sink and start playing.
    fn start_current(&mut self, offset_secs: u32) -> CoreResult<()> {
        let track = self
            .current()
            .ok_or_else(|| CoreError::Playback("queue is empty".into()))?
            .clone();
        self.sink.play(&track, offset_secs)?;
        self.sink.set_volume(self.volume)?;
        self.position_secs = offset_secs;
        self.state = PlaybackState::Playing;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(id: &str, title: &str) -> Track {
        Track {
            id: id.into(),
            title: title.into(),
            artist: "Tester".into(),
            album: None,
            genre: None,
            path: format!("/tmp/{id}.mp3"),
            duration_secs: 120,
            track_number: None,
            year: None,
            file_format: Some("mp3".into()),
        }
    }

    fn engine() -> PlaybackEngine<NullAudioSink> {
        PlaybackEngine::new(NullAudioSink::new())
    }

    #[test]
    fn play_pause_resume_stop_cycle() {
        let mut e = engine();
        assert_eq!(e.state(), PlaybackState::Stopped);

        e.play(track("t1", "Alpha")).unwrap();
        assert_eq!(e.state(), PlaybackState::Playing);
        assert_eq!(e.current().unwrap().id, "t1");

        e.pause().unwrap();
        assert_eq!(e.state(), PlaybackState::Paused);
        e.resume().unwrap();
        assert_eq!(e.state(), PlaybackState::Playing);

        e.stop().unwrap();
        assert_eq!(e.state(), PlaybackState::Stopped);
    }

    #[test]
    fn play_rejects_track_without_path() {
        let mut e = engine();
        let mut t = track("t1", "Alpha");
        t.path.clear();
        assert!(e.play(t).is_err());
    }

    #[test]
    fn queue_play_next_previous_off() {
        let mut e = engine();
        e.play_queue(vec![track("a", "A"), track("b", "B"), track("c", "C")])
            .unwrap();
        assert_eq!(e.current().unwrap().id, "a");

        e.advance().unwrap();
        assert_eq!(e.current().unwrap().id, "b");

        e.advance().unwrap();
        assert_eq!(e.current().unwrap().id, "c");

        // end of queue with repeat off -> stops
        e.advance().unwrap();
        assert_eq!(e.state(), PlaybackState::Stopped);

        // previous from stopped is an error
        assert!(e.previous().is_err());
    }

    #[test]
    fn queue_repeat_all_wraps_around() {
        let mut e = engine();
        e.set_repeat(RepeatMode::All);
        e.play_queue(vec![track("a", "A"), track("b", "B")])
            .unwrap();
        e.advance().unwrap();
        assert_eq!(e.current().unwrap().id, "b");
        e.advance().unwrap();
        assert_eq!(e.current().unwrap().id, "a"); // wrapped
    }

    #[test]
    fn queue_repeat_one_stays_on_same_track() {
        let mut e = engine();
        e.set_repeat(RepeatMode::One);
        e.play_queue(vec![track("a", "A"), track("b", "B")])
            .unwrap();
        e.advance().unwrap();
        assert_eq!(e.current().unwrap().id, "a");
    }

    #[test]
    fn volume_clamps_to_unit_range() {
        let mut e = engine();
        e.set_volume(5.0).unwrap();
        assert!((e.volume() - 1.0).abs() < 1e-9);
        e.set_volume(-1.0).unwrap();
        assert!(e.volume().abs() < 1e-9);
    }

    #[test]
    fn seek_updates_position() {
        let mut e = engine();
        e.play(track("t1", "X")).unwrap();
        e.seek(45).unwrap();
        assert_eq!(e.position_secs(), 45);
        assert_eq!(e.sink.position_secs(), 45);
    }

    #[test]
    fn previous_at_start_restarts_current() {
        let mut e = engine();
        e.play_queue(vec![track("a", "A"), track("b", "B")])
            .unwrap();
        e.seek(30).unwrap();
        e.previous().unwrap();
        assert_eq!(e.current().unwrap().id, "a");
        assert_eq!(e.position_secs(), 0);
    }

    #[test]
    fn clear_queue_resets_state() {
        let mut e = engine();
        e.play_queue(vec![track("a", "A")]).unwrap();
        e.clear_queue();
        assert_eq!(e.state(), PlaybackState::Stopped);
        assert!(e.current().is_none());
        assert!(e.queue().is_empty());
    }

    #[test]
    fn status_snapshot_is_consistent() {
        let mut e = engine();
        e.play_queue(vec![track("a", "A")]).unwrap();
        let s = e.status();
        assert_eq!(s.state, PlaybackState::Playing);
        assert_eq!(s.current_track.as_ref().unwrap().id, "a");
        assert_eq!(s.queue_length, 1);
        assert_eq!(s.queue_index, Some(0));
        assert!((s.volume - 1.0).abs() < 1e-9);
    }
}
