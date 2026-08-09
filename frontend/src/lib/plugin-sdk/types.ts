/**
 * EasyMusic Plugin SDK — TypeScript types shared between the host runtime
 * and plugin authors. Mirrors the Rust `easy-music-plugin-sdk` crate.
 */

/** Plugin engine type. Currently only "js" is supported. */
export type PluginEngine = "js" | "wasm";

/** Capabilities a plugin can request in its manifest. */
export type Permission =
  | "library:read"
  | "playback:control"
  | "network:fetch"
  | "ui:panel"
  | "audio:transform"
  | "playlist:access";

/** All hooks/events a plugin can subscribe to. */
export type PluginHookName =
  | "onPluginLoaded"
  | "onTrackChanged"
  | "onPlaybackStateChanged"
  | "onLibraryScanned"
  | "customUIPanel"
  | "audioTransform";

/** Plugin manifest — parsed from `plugin.json` on the Rust side. */
export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
  engine: PluginEngine;
  entry: string;
  permissions: Permission[];
  hooks: PluginHookName[];
  min_app_version?: string;
}

/** Runtime status of a plugin (from Rust host). */
export type PluginStatus = "enabled" | "disabled" | "error";

/** Plugin info DTO returned by Tauri commands. */
export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
  status: PluginStatus;
  error?: string;
  permissions: string[];
  hooks: string[];
}

/** A track in the library (from easy-music-core models). */
export interface Track {
  id: string;
  title: string;
  artist: string;
  album?: string;
  path: string;
  duration_secs: number;
}

/** Payload for onPlaybackStateChanged. */
export interface PlaybackStatePayload {
  state: "stopped" | "playing" | "paused";
  position_secs: number;
}

/** Payload for onLibraryScanned. */
export interface LibraryScannedPayload {
  track_count: number;
}

/** Payload for customUIPanel. */
export interface CustomUIPanelPayload {
  container: HTMLElement;
}

/** Payload for audioTransform. */
export interface AudioTransformPayload {
  channelData: Float32Array;
  sampleRate: number;
}

/** API surface granted to a plugin, scoped by its declared permissions. */
export interface PluginAPI {
  /** Plugin manifest (read-only). */
  manifest: PluginManifest;

  /** Log a message to the host console with plugin prefix. */
  log(...args: unknown[]): void;

  /** Get the current track (requires library:read). */
  getCurrentTrack?(): Track | null;

  /** Fetch a URL through the host proxy (requires network:fetch). */
  proxiedFetch?(url: string, opts?: { method?: string; headers?: Record<string, string> }): Promise<string>;

  /** Dispatch a custom event the host listens for (requires playback:control). */
  playbackControl?(action: "play" | "pause" | "stop"): void;
}

/**
 * The contract a plugin's default export must implement.
 * Plugin authors write an ES module that default-exports an object
 * implementing this interface.
 */
export interface EasyMusicPlugin {
  /** Called once when the plugin is loaded. */
  onLoad?(api: PluginAPI): void | Promise<void>;

  /** Called when the current track changes. */
  onTrackChanged?(track: Track): void;

  /** Called on play/pause/stop transitions. */
  onPlaybackStateChanged?(payload: PlaybackStatePayload): void;

  /** Called when a library scan completes. */
  onLibraryScanned?(payload: LibraryScannedPayload): void;

  /** Called to render a custom UI panel. */
  customUIPanel?(container: HTMLElement): void;

  /** Called for real-time audio processing. */
  audioTransform?(payload: AudioTransformPayload): Float32Array | void;

  /** Called when the plugin is being unloaded. */
  onUnload?(): void;
}
