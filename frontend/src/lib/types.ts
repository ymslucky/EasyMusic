/**
 * Shared TypeScript types — mirror the Rust structs in crates/easy-music-core.
 *
 * Keep these in lock-step with the `easy_music_core::models::*` definitions so
 * the Tauri IPC layer serializes/deserializes without manual mapping.
 */

/** A single audio track. Mirrors `easy_music_core::models::Track`. */
export interface Track {
  id: string;
  title: string;
  artist: string;
  album: string | null;
  genre: string | null;
  /** Absolute path on disk. */
  path: string;
  /** Duration in seconds (0 if unknown). */
  duration_secs: number;
  /** 1-based track number within the album, if tagged. */
  track_number: number | null;
  /** Release year, if tagged. */
  year: number | null;
  /** File extension (e.g. "mp3", "flac"). */
  file_format: string | null;
}

/** An album — a logical grouping of tracks. Mirrors `Album`. */
export interface Album {
  id: string;
  title: string;
  artist: string | null;
  year: number | null;
  genre: string | null;
  track_count: number;
}

/** An artist — derived from distinct track.artist values. Mirrors `Artist`. */
export interface Artist {
  id: string;
  name: string;
  album_count: number;
  track_count: number;
}

/** A user-created playlist (metadata only). Mirrors `Playlist`. */
export interface Playlist {
  id: string;
  name: string;
  track_count: number;
  /** ISO-8601 creation timestamp. */
  created_at: string;
}

/** A playlist with its full track list — returned by playlist_get. */
export interface PlaylistWithTracks {
  playlist: Playlist;
  tracks: Track[];
}

/** Transport state of the playback engine. Mirrors `PlaybackState` (camelCase serde). */
export type PlaybackState = "Stopped" | "Playing" | "Paused";

/** Repeat mode for queue playback. Mirrors `RepeatMode` (lowercase serde). */
export type RepeatMode = "off" | "all" | "one";

/** Snapshot of playback status returned by playback_status. Mirrors `PlaybackStatus`. */
export interface PlaybackStatus {
  state: PlaybackState;
  track_id: string | null;
  position_secs: number;
  duration_secs: number;
  volume: number;
}

/** Aggregate statistics about the entire library. Mirrors `LibraryMetadata`. */
export interface LibraryMetadata {
  total_tracks: number;
  total_albums: number;
  total_artists: number;
  total_playlists: number;
  total_duration_secs: number;
  last_scanned: string | null;
}

/** Result of a library scan. Mirrors `ScanResult`. */
export interface ScanResult {
  scanned_files: number;
  added: number;
  updated: number;
  skipped: number;
  errors: number;
}

/** Filtering criteria for tracks_filter. Mirrors `TrackFilter`. */
export interface TrackFilter {
  artist?: string | null;
  album?: string | null;
  genre?: string | null;
  min_duration_secs?: number | null;
  max_duration_secs?: number | null;
}

// --- Frontend-only types (not in Rust backend, browser mock only) -----------

/** A configured library directory on disk. */
export interface LibraryDirectory {
  id: string;
  path: string;
  label: string;
  enabled: boolean;
}

/** A loaded plugin. */
export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  /** Lifecycle status — "enabled" plugins are active, "disabled" are loaded
   *  but off, "error" means the plugin failed to load. */
  status: "enabled" | "disabled" | "error";
  /** Backwards-compat boolean — derived from status. */
  enabled: boolean;
  description: string;
  author?: string;
  /** Error message if status === "error". */
  error?: string;
  /** Hook names this plugin registers (e.g. ["on_track_change"]). */
  hooks: string[];
  /** Permission scopes the plugin requests. */
  permissions: string[];
}
