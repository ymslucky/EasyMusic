/**
 * lib/api.ts — typed Tauri command wrapper.
 *
 * Every Rust command is wrapped here so call sites get:
 *   - TypeScript types for args and return values
 *   - A graceful in-browser fallback (mock data) so the UI is fully
 *     interactive when `next dev` is opened outside Tauri.
 *
 * Backend command inventory (src-tauri/src/commands.rs):
 *   greet(name) -> string
 *   library_open_db(db_path) -> ()
 *   library_scan(root) -> ScanResult
 *   library_metadata() -> LibraryMetadata
 *   tracks_all() -> Vec<Track>
 *   track_get(id) -> Option<Track>
 *   tracks_search(query) -> Vec<Track>
 *   tracks_filter(filter) -> Vec<Track>
 *   albums_all() -> Vec<Album>
 *   artists_all() -> Vec<Artist>
 *   playlist_create(name) -> Playlist
 *   playlist_rename(id, new_name) -> ()
 *   playlist_delete(id) -> ()
 *   playlists_all() -> Vec<Playlist>
 *   playlist_get(id) -> PlaylistWithTracks
 *   playlist_add_track(playlist_id, track_id) -> ()
 *   playlist_remove_track(playlist_id, track_id) -> ()
 *   playback_status() -> PlaybackStatus
 *   playback_play_track(track) -> PlaybackStatus
 *   playback_play_queue(tracks) -> PlaybackStatus
 *   playback_pause() / playback_resume() / playback_stop() -> PlaybackStatus
 *   playback_seek(secs) -> PlaybackStatus
 *   playback_set_volume(volume) -> PlaybackStatus
 *   playback_next() / playback_previous() -> PlaybackStatus
 *   playback_set_repeat(mode) / playback_toggle_shuffle() -> PlaybackStatus
 */

import { isTauri } from "@tauri-apps/api/core";
import type {
  Album,
  Artist,
  LibraryDirectory,
  LibraryMetadata,
  Playlist,
  PlaylistWithTracks,
  PlaybackState,
  PlaybackStatus,
  PluginInfo,
  RepeatMode,
  ScanResult,
  Track,
  TrackFilter,
} from "./types";
import { genId } from "./utils";

// --- Tauri command dispatcher -----------------------------------------------

let cachedInvoke: typeof import("@tauri-apps/api/core").invoke | null = null;

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (cachedInvoke === null) {
    const mod = await import("@tauri-apps/api/core");
    cachedInvoke = mod.invoke;
  }
  return cachedInvoke<T>(cmd, args);
}

export const isTauriEnv = (): boolean => isTauri();

// --- Mock data (browser-only demo state) ------------------------------------

function makeMockTracks(): Track[] {
  const artists = [
    "Lunar Drift", "Echo Park", "Neon Tigers", "Sage & Stone",
    "Violet Hour", "Midnight Cartographer", "Paper Cities", "Golden Ratio",
  ];
  const albums = [
    "Aurora", "Static Bloom", "Paper Lanterns", "Tidewater",
    "Slow Light", "Glasshouse", "Outlines", "Northbound",
  ];
  const titles = [
    "Pale Horizon", "Cardinal", "Slow Tide", "Ironwood", "Telephone",
    "Northbound", "Saltwater", "Hold the Line", "Embers", "Cinder",
  ];
  const genres = ["Indie", "Electronic", "Ambient", "Rock"];
  return Array.from({ length: 48 }, (_, i) => {
    const artist = artists[i % artists.length]!;
    const album = albums[i % albums.length]!;
    const title = titles[i % titles.length]!;
    return {
      id: `mock_${i}`,
      title: `${title}${i >= titles.length ? " " + (Math.floor(i / titles.length) + 1) : ""}`,
      artist,
      album,
      genre: genres[i % genres.length]!,
      path: `/tmp/mock/${i}.flac`,
      duration_secs: 120 + ((i * 37) % 300),
      track_number: (i % 12) + 1,
      year: 2020 + (i % 5),
      file_format: "flac",
    } satisfies Track;
  });
}

const mockTracks = makeMockTracks();

const mockPlaylists: Playlist[] = [
  {
    id: "pl_favorites",
    name: "Favorites",
    track_count: 3,
    created_at: new Date(Date.now() - 86400000 * 7).toISOString(),
  },
  {
    id: "pl_focus",
    name: "Focus",
    track_count: 4,
    created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
  },
];

// Mock playlist track associations (browser only).
const mockPlaylistTracks: Record<string, string[]> = {
  pl_favorites: ["mock_0", "mock_3", "mock_7"],
  pl_focus: ["mock_1", "mock_2", "mock_5", "mock_9"],
};

const mockLibraryDirs: LibraryDirectory[] = [
  { id: "dir_1", path: "/home/user/Music", label: "Music", enabled: true },
  { id: "dir_2", path: "/mnt/library/flac", label: "FLAC Library", enabled: true },
];

const mockPlugins: PluginInfo[] = [
  { id: "lyrics", name: "Lyrics Fetcher", version: "0.1.0", status: "enabled", enabled: true, description: "Fetches and syncs time-synced lyrics.", author: "EasyMusic", hooks: ["on_track_change", "on_playback_start"], permissions: ["network"] },
  { id: "scrobble", name: "Last.fm Scrobbler", version: "0.2.1", status: "disabled", enabled: false, description: "Scrobbles played tracks to Last.fm.", author: "Community", hooks: ["on_playback_complete"], permissions: ["network", "api_key"] },
  { id: "equalizer", name: "Parametric EQ", version: "0.0.3", status: "disabled", enabled: false, description: "10-band parametric equalizer.", hooks: ["on_audio_process"], permissions: ["audio"] },
];

// In-memory mock playback state so the browser demo responds to every control.
const mockPlayback = {
  state: "Stopped" as PlaybackState,
  trackId: null as string | null,
  position: 0,
  volume: 0.8,
  queue: [] as Track[],
  queueIndex: -1,
  repeat: "off" as RepeatMode,
  shuffle: false,
};

// --- Public typed API --------------------------------------------------------

export const api = {
  isTauri: isTauriEnv,

  // --- Greeting ----------------------------------------------------------

  async greet(name: string): Promise<string> {
    if (!isTauriEnv()) return `Hello, ${name}! (browser preview)`;
    return invoke<string>("greet", { name });
  },

  // --- Library lifecycle -------------------------------------------------

  async libraryScan(root: string): Promise<ScanResult> {
    if (!isTauriEnv()) {
      return { scanned_files: 0, added: mockTracks.length, updated: 0, skipped: 0, errors: 0 };
    }
    return invoke<ScanResult>("library_scan", { root });
  },

  async libraryMetadata(): Promise<LibraryMetadata> {
    if (!isTauriEnv()) {
      return {
        total_tracks: mockTracks.length,
        total_albums: new Set(mockTracks.map((t) => t.album)).size,
        total_artists: new Set(mockTracks.map((t) => t.artist)).size,
        total_playlists: mockPlaylists.length,
        total_duration_secs: mockTracks.reduce((s, t) => s + t.duration_secs, 0),
        last_scanned: null,
      };
    }
    return invoke<LibraryMetadata>("library_metadata");
  },

  // --- Tracks ------------------------------------------------------------

  async tracksAll(): Promise<Track[]> {
    if (!isTauriEnv()) return mockTracks.slice();
    return invoke<Track[]>("tracks_all");
  },

  async trackGet(id: string): Promise<Track | null> {
    if (!isTauriEnv()) return mockTracks.find((t) => t.id === id) ?? null;
    return invoke<Track | null>("track_get", { id });
  },

  async tracksSearch(query: string): Promise<Track[]> {
    if (!isTauriEnv()) {
      const q = query.toLowerCase();
      return mockTracks.filter(
        (t) =>
          t.title.toLowerCase().includes(q) ||
          t.artist.toLowerCase().includes(q) ||
          (t.album ?? "").toLowerCase().includes(q),
      );
    }
    return invoke<Track[]>("tracks_search", { query });
  },

  async tracksFilter(filter: TrackFilter): Promise<Track[]> {
    if (!isTauriEnv()) {
      return mockTracks.filter((t) => {
        if (filter.artist && t.artist !== filter.artist) return false;
        if (filter.album && t.album !== filter.album) return false;
        if (filter.genre && t.genre !== filter.genre) return false;
        if (filter.min_duration_secs != null && t.duration_secs < filter.min_duration_secs) return false;
        if (filter.max_duration_secs != null && t.duration_secs > filter.max_duration_secs) return false;
        return true;
      });
    }
    return invoke<Track[]>("tracks_filter", { filter });
  },

  // --- Albums / Artists --------------------------------------------------

  async albumsAll(): Promise<Album[]> {
    if (!isTauriEnv()) {
      const map = new Map<string, Album>();
      for (const t of mockTracks) {
        const key = t.album ?? "Unknown Album";
        let a = map.get(key);
        if (!a) {
          a = {
            id: `album_${key.toLowerCase().replace(/\s+/g, "_")}`,
            title: key,
            artist: t.artist,
            year: t.year,
            genre: t.genre,
            track_count: 0,
          };
          map.set(key, a);
        }
        a.track_count++;
      }
      return Array.from(map.values()).sort((a, b) => a.title.localeCompare(b.title));
    }
    return invoke<Album[]>("albums_all");
  },

  async artistsAll(): Promise<Artist[]> {
    if (!isTauriEnv()) {
      const map = new Map<string, Artist>();
      for (const t of mockTracks) {
        let a = map.get(t.artist);
        if (!a) {
          a = { id: `artist_${t.artist.toLowerCase().replace(/\s+/g, "_")}`, name: t.artist, album_count: 0, track_count: 0 };
          map.set(t.artist, a);
        }
        a.track_count++;
      }
      // count albums per artist
      for (const a of map.values()) {
        a.album_count = new Set(mockTracks.filter((t) => t.artist === a.name).map((t) => t.album)).size;
      }
      return Array.from(map.values()).sort((a, b) => a.name.localeCompare(b.name));
    }
    return invoke<Artist[]>("artists_all");
  },

  // --- Playlists ---------------------------------------------------------

  async playlistsAll(): Promise<Playlist[]> {
    if (!isTauriEnv()) return mockPlaylists.slice();
    return invoke<Playlist[]>("playlists_all");
  },

  async playlistCreate(name: string): Promise<Playlist> {
    if (!isTauriEnv()) {
      const pl: Playlist = {
        id: genId("pl"),
        name,
        track_count: 0,
        created_at: new Date().toISOString(),
      };
      mockPlaylists.push(pl);
      mockPlaylistTracks[pl.id] = [];
      return pl;
    }
    return invoke<Playlist>("playlist_create", { name });
  },

  async playlistRename(id: string, newName: string): Promise<void> {
    if (!isTauriEnv()) {
      const pl = mockPlaylists.find((p) => p.id === id);
      if (pl) pl.name = newName;
      return;
    }
    return invoke<void>("playlist_rename", { id, newName });
  },

  async playlistDelete(id: string): Promise<void> {
    if (!isTauriEnv()) {
      const idx = mockPlaylists.findIndex((p) => p.id === id);
      if (idx >= 0) mockPlaylists.splice(idx, 1);
      delete mockPlaylistTracks[id];
      return;
    }
    return invoke<void>("playlist_delete", { id });
  },

  async playlistGet(id: string): Promise<PlaylistWithTracks> {
    if (!isTauriEnv()) {
      const pl = mockPlaylists.find((p) => p.id === id);
      const trackIds = mockPlaylistTracks[id] ?? [];
      const tracks = trackIds.map((tid) => mockTracks.find((t) => t.id === tid)).filter(Boolean) as Track[];
      return {
        playlist: pl ?? { id, name: "Unknown", track_count: 0, created_at: new Date().toISOString() },
        tracks,
      };
    }
    return invoke<PlaylistWithTracks>("playlist_get", { id });
  },

  async playlistAddTrack(playlistId: string, trackId: string): Promise<void> {
    if (!isTauriEnv()) {
      const list = mockPlaylistTracks[playlistId] ?? (mockPlaylistTracks[playlistId] = []);
      if (!list.includes(trackId)) {
        list.push(trackId);
        const pl = mockPlaylists.find((p) => p.id === playlistId);
        if (pl) pl.track_count = list.length;
      }
      return;
    }
    return invoke<void>("playlist_add_track", { playlistId, trackId });
  },

  async playlistRemoveTrack(playlistId: string, trackId: string): Promise<void> {
    if (!isTauriEnv()) {
      const list = mockPlaylistTracks[playlistId] ?? [];
      const idx = list.indexOf(trackId);
      if (idx >= 0) {
        list.splice(idx, 1);
        const pl = mockPlaylists.find((p) => p.id === playlistId);
        if (pl) pl.track_count = list.length;
      }
      return;
    }
    return invoke<void>("playlist_remove_track", { playlistId, trackId });
  },

  /** Reorder tracks in a playlist by sending the full new order. */
  async playlistReorder(playlistId: string, trackIds: string[]): Promise<void> {
    if (!isTauriEnv()) {
      mockPlaylistTracks[playlistId] = trackIds.slice();
      const pl = mockPlaylists.find((p) => p.id === playlistId);
      if (pl) pl.track_count = trackIds.length;
      return;
    }
    // Backend doesn't have a bulk reorder command yet. For now we remove all
    // and re-add in order — this is correct but not optimal. A future task
    // should add `playlist_set_tracks`.
    const current = await invoke<PlaylistWithTracks>("playlist_get", { id: playlistId });
    for (const t of current.tracks) {
      await invoke<void>("playlist_remove_track", { playlistId, trackId: t.id });
    }
    for (const tid of trackIds) {
      await invoke<void>("playlist_add_track", { playlistId, trackId: tid });
    }
  },

  // --- Playback ----------------------------------------------------------

  async playbackStatus(): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      const track = mockTracks.find((t) => t.id === mockPlayback.trackId);
      return {
        state: mockPlayback.state,
        track_id: mockPlayback.trackId,
        position_secs: mockPlayback.position,
        duration_secs: track?.duration_secs ?? 0,
        volume: mockPlayback.volume,
        repeat: mockPlayback.repeat,
        shuffle: mockPlayback.shuffle,
        queue_length: mockPlayback.queue.length,
        queue_index: mockPlayback.queueIndex >= 0 ? mockPlayback.queueIndex : null,
      };
    }
    return invoke<PlaybackStatus>("playback_status");
  },

  async playbackPlayTrack(track: Track): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.state = "Playing";
      mockPlayback.trackId = track.id;
      mockPlayback.position = 0;
      mockPlayback.queue = [track];
      mockPlayback.queueIndex = 0;
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_play_track", { track });
  },

  async playbackPlayQueue(tracks: Track[], startIndex = 0): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.queue = tracks.slice();
      mockPlayback.queueIndex = Math.max(0, Math.min(startIndex, tracks.length - 1));
      const t = tracks[mockPlayback.queueIndex];
      mockPlayback.trackId = t?.id ?? null;
      mockPlayback.state = t ? "Playing" : "Stopped";
      mockPlayback.position = 0;
      return mockStatus();
    }
    // Backend play_queue starts at index 0; for startIndex > 0 we slice.
    const sliced = tracks.slice(startIndex);
    return invoke<PlaybackStatus>("playback_play_queue", { tracks: sliced });
  },

  async playbackPause(): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.state = "Paused";
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_pause");
  },

  async playbackResume(): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.state = "Playing";
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_resume");
  },

  async playbackStop(): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.state = "Stopped";
      mockPlayback.position = 0;
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_stop");
  },

  async playbackSeek(secs: number): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.position = secs;
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_seek", { secs });
  },

  async playbackSetVolume(volume: number): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.volume = Math.max(0, Math.min(1, volume));
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_set_volume", { volume });
  },

  async playbackNext(): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      if (mockPlayback.queue.length > 0) {
        mockPlayback.queueIndex++;
        if (mockPlayback.queueIndex >= mockPlayback.queue.length) {
          if (mockPlayback.repeat === "all") mockPlayback.queueIndex = 0;
          else {
            mockPlayback.state = "Stopped";
            mockPlayback.position = 0;
            return mockStatus();
          }
        }
        const t = mockPlayback.queue[mockPlayback.queueIndex];
        mockPlayback.trackId = t?.id ?? null;
        mockPlayback.position = 0;
        if (t) mockPlayback.state = "Playing";
      }
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_next");
  },

  async playbackPrevious(): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      if (mockPlayback.queue.length > 0 && mockPlayback.position > 3) {
        mockPlayback.position = 0;
        return mockStatus();
      }
      if (mockPlayback.queue.length > 0) {
        mockPlayback.queueIndex = Math.max(0, mockPlayback.queueIndex - 1);
        const t = mockPlayback.queue[mockPlayback.queueIndex];
        mockPlayback.trackId = t?.id ?? null;
        mockPlayback.position = 0;
        if (t) mockPlayback.state = "Playing";
      }
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_previous");
  },

  async playbackSetRepeat(mode: RepeatMode): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.repeat = mode;
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_set_repeat", { mode });
  },

  async playbackToggleShuffle(): Promise<PlaybackStatus> {
    if (!isTauriEnv()) {
      mockPlayback.shuffle = !mockPlayback.shuffle;
      return mockStatus();
    }
    return invoke<PlaybackStatus>("playback_toggle_shuffle");
  },

  /** Advance mock position by dt seconds (called by the store tick, browser only). */
  _mockTick(dtSecs: number): PlaybackStatus | null {
    if (mockPlayback.state !== "Playing" || !mockPlayback.trackId) return null;
    const track = mockTracks.find((t) => t.id === mockPlayback.trackId);
    const dur = track?.duration_secs ?? 0;
    mockPlayback.position = Math.min(dur, mockPlayback.position + dtSecs);
    if (dur > 0 && mockPlayback.position >= dur) {
      // auto-advance
      const s = api.playbackNext();
      void s;
    }
    return mockStatus();
  },

  // --- Library directories (mock only — no backend command yet) ----------

  async listLibraryDirs(): Promise<LibraryDirectory[]> {
    return mockLibraryDirs.slice();
  },

  async addLibraryDir(path: string, label: string): Promise<LibraryDirectory> {
    const dir: LibraryDirectory = { id: genId("dir"), path, label, enabled: true };
    mockLibraryDirs.push(dir);
    return dir;
  },

  async removeLibraryDir(id: string): Promise<void> {
    const idx = mockLibraryDirs.findIndex((d) => d.id === id);
    if (idx >= 0) mockLibraryDirs.splice(idx, 1);
  },

  // --- Plugins ----------------------------------------------------------
  //
  // Tauri mode talks to the Rust PluginManager commands
  // (src-tauri/src/plugin_commands.rs); the browser fallback keeps an
  // in-memory demo list so the UI stays interactive outside the desktop app.

  async listPlugins(): Promise<PluginInfo[]> {
    if (!isTauriEnv()) return mockPlugins.slice();
    return invoke<PluginInfo[]>("list_plugins");
  },

  async togglePlugin(id: string, enabled: boolean): Promise<void> {
    if (!isTauriEnv()) {
      const p = mockPlugins.find((x) => x.id === id);
      if (p) {
        p.enabled = enabled;
        p.status = enabled ? "enabled" : "disabled";
      }
      return;
    }
    return invoke<void>(enabled ? "enable_plugin" : "disable_plugin", { id });
  },

  async installPlugin(path: string): Promise<PluginInfo> {
    if (!isTauriEnv()) {
      // Mock: create a placeholder plugin entry.
      const p: PluginInfo = {
        id: genId("plugin"),
        name: path.split("/").pop() ?? "New Plugin",
        version: "0.0.1",
        status: "disabled",
        enabled: false,
        description: `Installed from ${path}`,
        hooks: [],
        permissions: [],
      };
      mockPlugins.push(p);
      return p;
    }
    return invoke<PluginInfo>("install_plugin", { path });
  },

  async uninstallPlugin(id: string): Promise<void> {
    if (!isTauriEnv()) {
      const idx = mockPlugins.findIndex((x) => x.id === id);
      if (idx >= 0) mockPlugins.splice(idx, 1);
      return;
    }
    return invoke<void>("uninstall_plugin", { id });
  },

  async reloadPlugins(): Promise<PluginInfo[]> {
    if (!isTauriEnv()) return mockPlugins.slice();
    await invoke<void>("reload_plugins");
    return invoke<PluginInfo[]>("list_plugins");
  },
};

function mockStatus(): PlaybackStatus {
  const track = mockTracks.find((t) => t.id === mockPlayback.trackId);
  return {
    state: mockPlayback.state,
    track_id: mockPlayback.trackId,
    position_secs: mockPlayback.position,
    duration_secs: track?.duration_secs ?? 0,
    volume: mockPlayback.volume,
    repeat: mockPlayback.repeat,
    shuffle: mockPlayback.shuffle,
    queue_length: mockPlayback.queue.length,
    queue_index: mockPlayback.queueIndex >= 0 ? mockPlayback.queueIndex : null,
  };
}

export type Api = typeof api;
