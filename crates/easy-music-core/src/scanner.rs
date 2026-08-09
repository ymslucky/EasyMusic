//! Library scanner — recursively walks a directory, extracts metadata via
//! the `lofty` crate (ID3, Vorbis/FLAC, MP4, etc.), and produces `Track`s
//! ready for DB insertion.

use std::path::{Path, PathBuf};

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::Accessor;
use walkdir::WalkDir;

use crate::error::{CoreError, CoreResult};
use crate::models::Track;

/// Audio extensions we attempt to parse. Anything else is silently skipped.
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "wma",
];

/// Options for a scan. Currently just a root path, but kept as a struct
/// so future knobs (follow symlinks, max depth, extension overrides) can
/// be added without breaking the API.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub root: PathBuf,
}

impl ScanOptions {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

/// Scan the configured directory and return every parseable track found.
///
/// Unparseable files are counted as errors but do not abort the scan; the
/// caller receives the aggregate [`ScanResult`] via the DB upsert.
pub fn scan_directory(opts: &ScanOptions) -> CoreResult<Vec<Track>> {
    let root = &opts.root;
    if !root.exists() {
        return Err(CoreError::NotFound(format!(
            "scan root does not exist: {}",
            root.display()
        )));
    }
    if !root.is_dir() {
        return Err(CoreError::Invalid(format!(
            "scan root is not a directory: {}",
            root.display()
        )));
    }

    let mut tracks = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip unreadable entries
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !is_supported(path) {
            continue;
        }
        match parse_file(path) {
            Ok(Some(track)) => tracks.push(track),
            Ok(None) => {}
            Err(_) => {} // tag parse failure — skip, don't abort
        }
    }
    Ok(tracks)
}

/// Return true if the path's extension is one we try to parse.
fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|supported| supported.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false)
}

/// Parse a single audio file into a `Track`. Returns `Ok(None)` if the file
/// is not recognized as audio at all.
pub fn parse_file(path: &Path) -> CoreResult<Option<Track>> {
    let tagged = match read_from_path(path) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    Ok(Some(tagged_file_to_track(path, &tagged)))
}

/// Build a `Track` from a parsed `TaggedFile`.
fn tagged_file_to_track(path: &Path, tagged: &lofty::file::TaggedFile) -> Track {
    let tag = tagged.primary_tag();

    let title = tag
        .and_then(|t| t.title().map(|s| s.to_string()))
        .unwrap_or_else(|| {
            // Fall back to the file stem if there's no title tag.
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        });

    let artist = tag
        .and_then(|t| t.artist().map(|s| s.to_string()))
        .unwrap_or_else(|| "Unknown Artist".to_string());

    let album = tag.and_then(|t| t.album().map(|s| s.to_string()));
    let genre = tag.and_then(|t| t.genre().map(|s| s.to_string()));

    let duration_secs = tagged
        .properties()
        .duration()
        .as_secs()
        .min(u32::MAX as u64) as u32;

    let track_number = tag.and_then(|t| t.track());
    let year = tag.and_then(|t| t.year());

    let file_format = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    Track {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        artist,
        album,
        genre,
        path: path.to_string_lossy().into_owned(),
        duration_secs,
        track_number,
        year,
        file_format,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::LibraryManager;

    #[test]
    fn is_supported_recognizes_common_formats() {
        assert!(is_supported(Path::new("/x/y.mp3")));
        assert!(is_supported(Path::new("/x/y.FLAC")));
        assert!(!is_supported(Path::new("/x/y.txt")));
        assert!(!is_supported(Path::new("/x/noext")));
    }

    /// Integration test: scan a real (silent, generated) WAV file end-to-end
    /// through the scanner + DB upsert, then verify it's retrievable.
    #[test]
    fn scan_and_index_wav_file() {
        // Create a temporary directory with a minimal WAV file.
        let tmp = std::env::temp_dir().join(format!(
            "easymusic_scan_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&tmp).unwrap();
        let wav_path = tmp.join("silence.wav");
        // Write a 1-second silent 44100Hz mono 16-bit PCM WAV.
        let sample_rate: u32 = 44100;
        let num_samples: usize = sample_rate as usize;
        let data_size = num_samples * 2;
        let mut buf = Vec::with_capacity(44 + data_size);
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&(36 + data_size as u32).to_le_bytes());
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
        buf.extend_from_slice(&1u16.to_le_bytes()); // mono
        buf.extend_from_slice(&sample_rate.to_le_bytes());
        buf.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
        buf.extend_from_slice(&2u16.to_le_bytes()); // block align
        buf.extend_from_slice(&16u16.to_le_bytes()); // bits/sample
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&(data_size as u32).to_le_bytes());
        buf.extend(std::iter::repeat(0u8).take(data_size));
        std::fs::write(&wav_path, &buf).unwrap();

        let lib = LibraryManager::open_memory().unwrap();
        let result = lib.scan_directory(&tmp).unwrap();
        assert_eq!(result.scanned_files, 1);
        assert_eq!(result.added, 1);
        assert_eq!(result.updated, 0);

        let tracks = lib.all_tracks().unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "silence");
        assert_eq!(tracks[0].artist, "Unknown Artist");
        assert_eq!(tracks[0].file_format.as_deref(), Some("wav"));
        assert!(tracks[0].duration_secs >= 1);

        // Re-scan: should update, not duplicate.
        let result2 = lib.scan_directory(&tmp).unwrap();
        assert_eq!(result2.scanned_files, 1);
        assert_eq!(result2.added, 0);
        assert_eq!(result2.updated, 1);
        assert_eq!(lib.all_tracks().unwrap().len(), 1);

        // Cleanup.
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn scan_nonexistent_root_errors() {
        let result = scan_directory(&ScanOptions::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_err());
    }
}
