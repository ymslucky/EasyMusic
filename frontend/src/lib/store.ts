/**
 * lib/store.ts — global application state via Zustand.
 *
 * Single store with slices for:
 *   - library:  track cache + sort/filter UI state
 *   - albums:   album/artist groupings (fetched from backend)
 *   - playlists: CRUD (with track associations cached locally)
 *   - playback: transport state, current track, position, queue, volume
 *   - settings: library dirs, theme, plugins
 *   - ui:       active view + selected ids
 *
 * The store is the only place that talks to `api`; components subscribe
 * to slices and call actions.
 */

"use client";

import { create } from "zustand";
import { api, isTauriEnv } from "./api";
import type {
  Album,
  Artist,
  LibraryDirectory,
  Playlist,
  PlaybackState,
  PluginInfo,
  RepeatMode,
  Track,
} from "./types";

// --- Sort / filter types -----------------------------------------------------

export type SortKey = "title" | "artist" | "album" | "duration_secs";
export type SortDir = "asc" | "desc";

export interface LibraryFilter {
  search: string;
  sortKey: SortKey;
  sortDir: SortDir;
}

// --- Store shape -------------------------------------------------------------

export interface AppState {
  // data
  tracks: Track[];
  tracksById: Record<string, Track>;
  albums: Album[];
  artists: Artist[];
  playlists: Playlist[];
  /** Track ids per playlist id (cached for DnD reordering). */
  playlistTracks: Record<string, string[]>;
  libraryDirs: LibraryDirectory[];
  plugins: PluginInfo[];

  // loading flags
  loading: boolean;
  error: string | null;

  // library UI
  filter: LibraryFilter;

  // playback
  playbackState: PlaybackState;
  currentTrackId: string | null;
  positionSecs: number;
  durationSecs: number;
  volume: number;
  queue: Track[];
  queueIndex: number;
  repeatMode: RepeatMode;
  shuffle: boolean;

  // navigation / UI
  view:
    | { name: "library" }
    | { name: "albums" }
    | { name: "album"; key: string }
    | { name: "artists" }
    | { name: "artist"; key: string }
    | { name: "playlists" }
    | { name: "playlist"; id: string }
    | { name: "settings" };

  // internal
  _tickHandle: ReturnType<typeof setInterval> | null;
  _initialized: boolean;

  // actions
  init: () => Promise<void>;
  refreshLibrary: () => Promise<void>;
  setFilter: (patch: Partial<LibraryFilter>) => void;
  navigate: (view: AppState["view"]) => void;

  // playback actions
  playTrack: (track: Track, queueContext?: Track[]) => void;
  togglePlay: () => void;
  next: () => void;
  prev: () => void;
  seek: (secs: number) => void;
  setVolume: (v: number) => void;
  toggleRepeat: () => void;
  toggleShuffle: () => void;
  onPositionTick: (dtSecs: number) => void;

  // playlist actions
  createPlaylist: (name: string) => Promise<Playlist | null>;
  deletePlaylist: (id: string) => Promise<void>;
  renamePlaylist: (id: string, name: string) => Promise<void>;
  loadPlaylistTracks: (id: string) => Promise<string[]>;
  reorderPlaylistTracks: (id: string, trackIds: string[]) => Promise<void>;

  // settings actions
  addLibraryDir: (path: string, label: string) => Promise<void>;
  removeLibraryDir: (id: string) => Promise<void>;
  togglePlugin: (id: string, enabled: boolean) => Promise<void>;
  reloadPlugins: () => Promise<void>;
  installPlugin: (path: string) => Promise<void>;
  uninstallPlugin: (id: string) => Promise<void>;

  // selectors
  getFilteredTracks: () => Track[];
}

// --- Helpers -----------------------------------------------------------------

function shuffleArray<T>(arr: T[]): T[] {
  const a = arr.slice();
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

// --- Store -------------------------------------------------------------------

export const useStore = create<AppState>((set, get) => ({
  tracks: [],
  tracksById: {},
  albums: [],
  artists: [],
  playlists: [],
  playlistTracks: {},
  libraryDirs: [],
  plugins: [],

  loading: false,
  error: null,

  filter: { search: "", sortKey: "title", sortDir: "asc" },

  playbackState: "Stopped",
  currentTrackId: null,
  positionSecs: 0,
  durationSecs: 0,
  volume: 0.8,
  queue: [],
  queueIndex: -1,
  repeatMode: "off",
  shuffle: false,

  view: { name: "library" },

  _tickHandle: null,
  _initialized: false,

  init: async () => {
    if (get()._initialized) return;
    set({ loading: true, error: null });
    try {
      await get().refreshLibrary();
      const [playlists, libraryDirs, plugins, albums, artists] = await Promise.all([
        api.playlistsAll(),
        api.listLibraryDirs(),
        api.listPlugins(),
        api.albumsAll(),
        api.artistsAll(),
      ]);
      // Cache track associations for each playlist
      const playlistTracks: Record<string, string[]> = {};
      for (const pl of playlists) {
        try {
          const withTracks = await api.playlistGet(pl.id);
          playlistTracks[pl.id] = withTracks.tracks.map((t) => t.id);
        } catch {
          playlistTracks[pl.id] = [];
        }
      }
      set({
        playlists,
        playlistTracks,
        libraryDirs,
        plugins,
        albums,
        artists,
        loading: false,
        _initialized: true,
      });

      // Start a position-ticking interval for smooth seek-bar movement.
      const handle = setInterval(() => {
        get().onPositionTick(1);
      }, 1000);
      set({ _tickHandle: handle });
    } catch (err) {
      set({ loading: false, error: String(err) });
    }
  },

  refreshLibrary: async () => {
    const tracks = await api.tracksAll();
    const tracksById: Record<string, Track> = {};
    for (const t of tracks) tracksById[t.id] = t;
    set({ tracks, tracksById });
  },

  setFilter: (patch) => set((s) => ({ filter: { ...s.filter, ...patch} })),

  navigate: (view) => set({ view }),

  // --- Playback -------------------------------------------------------------

  playTrack: (track, queueContext) => {
    const { queue, queueIndex } = get();
    let nextQueue = queue;
    let nextIdx = queueIndex;
    if (queueContext && queueContext.length > 0) {
      nextQueue = get().shuffle ? shuffleArray(queueContext) : queueContext.slice();
      nextIdx = nextQueue.findIndex((t) => t.id === track.id);
      if (nextIdx < 0) {
        nextQueue = [track, ...nextQueue];
        nextIdx = 0;
      }
    } else {
      nextQueue = [track];
      nextIdx = 0;
    }
    set({
      queue: nextQueue,
      queueIndex: nextIdx,
      currentTrackId: track.id,
      playbackState: "Playing",
      positionSecs: 0,
      durationSecs: track.duration_secs,
    });
    void api.playbackPlayQueue(nextQueue, nextIdx);
  },

  togglePlay: () => {
    const { playbackState } = get();
    if (playbackState === "Playing") {
      set({ playbackState: "Paused" });
      void api.playbackPause();
    } else if (playbackState === "Paused") {
      set({ playbackState: "Playing" });
      void api.playbackResume();
    }
  },

  next: () => {
    const { queue, queueIndex, repeatMode } = get();
    if (queue.length === 0) return;
    let idx = queueIndex + 1;
    if (idx >= queue.length) {
      if (repeatMode === "all") idx = 0;
      else {
        set({ playbackState: "Stopped", positionSecs: 0, queueIndex: queue.length });
        void api.playbackStop();
        return;
      }
    }
    const track = queue[idx];
    if (!track) return;
    set({
      queueIndex: idx,
      currentTrackId: track.id,
      playbackState: "Playing",
      positionSecs: 0,
      durationSecs: track.duration_secs,
    });
    void api.playbackNext();
  },

  prev: () => {
    const { queue, queueIndex, positionSecs } = get();
    if (queue.length === 0) return;
    if (positionSecs > 3) {
      set({ positionSecs: 0 });
      void api.playbackSeek(0);
      return;
    }
    const idx = Math.max(0, queueIndex - 1);
    const track = queue[idx];
    if (!track) return;
    set({
      queueIndex: idx,
      currentTrackId: track.id,
      playbackState: "Playing",
      positionSecs: 0,
      durationSecs: track.duration_secs,
    });
    void api.playbackPrevious();
  },

  seek: (secs) => {
    set({ positionSecs: secs });
    void api.playbackSeek(secs);
  },

  setVolume: (v) => {
    const vol = Math.max(0, Math.min(1, v));
    set({ volume: vol });
    void api.playbackSetVolume(vol);
  },

  toggleRepeat: () => {
    const newMode: RepeatMode =
      get().repeatMode === "off" ? "all" : get().repeatMode === "all" ? "one" : "off";
    set({ repeatMode: newMode });
    void api.playbackSetRepeat(newMode);
  },

  toggleShuffle: () => {
    const newShuffle = !get().shuffle;
    set({ shuffle: newShuffle });
    void api.playbackToggleShuffle();
  },

  onPositionTick: (dtSecs) => {
    const { playbackState, currentTrackId, tracksById } = get();
    if (playbackState !== "Playing" || !currentTrackId) return;
    const track = tracksById[currentTrackId];
    if (!track) return;
    const dur = track.duration_secs;

    if (!isTauriEnv()) {
      // Browser mock — advance locally and auto-advance on track end.
      const next = get().positionSecs + dtSecs;
      if (dur > 0 && next >= dur) {
        get().next();
        return;
      }
      set({ positionSecs: next });
      return;
    }

    // In Tauri — optimistically advance between backend status polls.
    set((s) => ({ positionSecs: Math.min(dur, s.positionSecs + dtSecs) }));
  },

  // --- Playlists ------------------------------------------------------------

  createPlaylist: async (name) => {
    try {
      const pl = await api.playlistCreate(name);
      set((s) => ({
        playlists: [...s.playlists, pl],
        playlistTracks: { ...s.playlistTracks, [pl.id]: [] },
      }));
      return pl;
    } catch (err) {
      set({ error: String(err) });
      return null;
    }
  },

  deletePlaylist: async (id) => {
    await api.playlistDelete(id);
    set((s) => {
      const playlistTracks = { ...s.playlistTracks };
      delete playlistTracks[id];
      return {
        playlists: s.playlists.filter((p) => p.id !== id),
        playlistTracks,
        view: s.view.name === "playlist" && s.view.id === id ? { name: "playlists" } : s.view,
      };
    });
  },

  renamePlaylist: async (id, name) => {
    await api.playlistRename(id, name);
    set((s) => ({
      playlists: s.playlists.map((p) => (p.id === id ? { ...p, name } : p)),
    }));
  },

  loadPlaylistTracks: async (id) => {
    const cached = get().playlistTracks[id];
    if (cached) return cached;
    try {
      const result = await api.playlistGet(id);
      const trackIds = result.tracks.map((t) => t.id);
      set((s) => ({ playlistTracks: { ...s.playlistTracks, [id]: trackIds } }));
      return trackIds;
    } catch {
      return [];
    }
  },

  reorderPlaylistTracks: async (id, trackIds) => {
    await api.playlistReorder(id, trackIds);
    set((s) => ({
      playlistTracks: { ...s.playlistTracks, [id]: trackIds },
      playlists: s.playlists.map((p) =>
        p.id === id ? { ...p, track_count: trackIds.length } : p,
      ),
    }));
  },

  // --- Settings -------------------------------------------------------------

  addLibraryDir: async (path, label) => {
    const dir = await api.addLibraryDir(path, label);
    set((s) => ({ libraryDirs: [...s.libraryDirs, dir] }));
  },

  removeLibraryDir: async (id) => {
    await api.removeLibraryDir(id);
    set((s) => ({ libraryDirs: s.libraryDirs.filter((d) => d.id !== id) }));
  },

  togglePlugin: async (id, enabled) => {
    await api.togglePlugin(id, enabled);
    set((s) => ({
      plugins: s.plugins.map((p) =>
        p.id === id
          ? { ...p, enabled, status: enabled ? "enabled" as const : "disabled" as const }
          : p,
      ),
    }));
  },

  reloadPlugins: async () => {
    const plugins = await api.reloadPlugins();
    set({ plugins });
  },

  installPlugin: async (path) => {
    const plugin = await api.installPlugin(path);
    set((s) => ({ plugins: [...s.plugins, plugin] }));
  },

  uninstallPlugin: async (id) => {
    await api.uninstallPlugin(id);
    set((s) => ({ plugins: s.plugins.filter((p) => p.id !== id) }));
  },

  // --- Selectors ------------------------------------------------------------

  getFilteredTracks: () => {
    const { tracks, filter } = get();
    let result = tracks;
    const q = filter.search.trim().toLowerCase();
    if (q) {
      result = result.filter(
        (t) =>
          t.title.toLowerCase().includes(q) ||
          t.artist.toLowerCase().includes(q) ||
          (t.album ?? "").toLowerCase().includes(q),
      );
    }
    const dir = filter.sortDir === "asc" ? 1 : -1;
    const key = filter.sortKey;
    return result.slice().sort((a, b) => {
      let av: string | number = "";
      let bv: string | number = "";
      if (key === "duration_secs") {
        av = a.duration_secs;
        bv = b.duration_secs;
      } else {
        av = (a[key] ?? "").toString().toLowerCase();
        bv = (b[key] ?? "").toString().toLowerCase();
      }
      if (av < bv) return -1 * dir;
      if (av > bv) return 1 * dir;
      return 0;
    });
  },
}));
