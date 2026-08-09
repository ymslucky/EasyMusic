# ADR-0001: Plugin System Architecture

**Status:** Accepted  
**Date:** 2026-08-10

## Context

EasyMusic needs an extensible plugin system so third-party developers can add
features: visualizers, lyrics fetchers, equalizers, new audio sources, custom UI
panels, and more. The app is a Tauri v2 + Next.js + Rust monorepo:

- **Frontend**: Next.js 16 (static export) + React 19, running inside Tauri's WebView
- **Backend**: Rust via Tauri commands, backed by `easy-music-core`
- **Existing surface**: `LibraryManager`, `PlaybackEngine`, two Tauri commands

## Decision: Hybrid JavaScript Plugin System

We adopt a **JavaScript-first hybrid plugin model**:

| Layer         | Responsibility                                                        |
|---------------|-----------------------------------------------------------------------|
| **Rust host** | Discover plugins on disk, validate manifests, manage enable/disable, persist state, expose metadata + entry-point source to the frontend via Tauri commands. |
| **JS runtime**| Load plugin entry-point scripts in the WebView, dispatch lifecycle/event hooks, render custom UI panels, apply audio transforms via Web Audio API. |

### Why JavaScript (not WebAssembly)?

We considered three options:

1. **WebAssembly components via `wasmtime`** — Sandboxed, language-agnostic, high
   performance. But: adds a heavy Rust dependency (`wasmtime` ~30s compile),
   requires a WASI/component toolchain for plugin authors, cannot render UI
   natively (needs a complex host-provided rendering bridge), and Tauri's WebView
   already provides a capable JS runtime. Overkill for the plugin types we need
   today (lyrics, visualizers, UI panels).

2. **Pure JS plugins loaded in the frontend** — Simple, rich UI ecosystem, natural
   fit for React. Concern: security (untrusted code). Mitigated by a
   permission-based manifest model and API surface that is opt-in per permission.

3. **Hybrid: WASM for audio transforms, JS for UI** — Theoretically ideal but
   doubles complexity. Audio transforms can use the Web Audio API
   (`AudioWorklet`) which is already performant. We can add WASM as an optional
   plugin format later without breaking the manifest schema.

**Decision: Option 2 (JS-first), designed so WASM can be added as an optional
`engine` later.** This delivers maximum value with minimum complexity and gives
plugin authors the easiest path (write a JS module, ship a `plugin.json`).

### Security model

- Plugins declare `permissions` in their manifest: `library:read`,
  `playback:control`, `network:fetch`, `ui:panel`, `audio:transform`.
- The Rust host validates the manifest and rejects plugins requesting unknown
  permissions.
- The frontend runtime checks permissions before granting API access to a plugin.
- Network access is gated behind `network:fetch` — the host proxies fetch calls
  through a Tauri command so plugins cannot make arbitrary requests without
  declaration.
- This is defense-in-depth, not a hard sandbox; plugins run in the same WebView
  origin. A future version can introduce Web Workers or `<iframe sandbox>` for
  stronger isolation.

## Manifest schema (`plugin.json`)

```json
{
  "id": "com.example.lyrics",
  "name": "Lyrics Display",
  "version": "1.0.0",
  "author": "Jane Doe",
  "description": "Fetches and displays synced lyrics.",
  "engine": "js",
  "entry": "index.js",
  "permissions": ["library:read", "network:fetch", "ui:panel"],
  "hooks": ["onTrackChanged", "customUIPanel"],
  "min_app_version": "0.1.0"
}
```

## Hook catalog

| Hook                   | When fired                          | Data payload                          |
|------------------------|-------------------------------------|---------------------------------------|
| `onPluginLoaded`       | Plugin registered in runtime        | Plugin context + API handle           |
| `onTrackChanged`       | Current track changes               | `Track` object                        |
| `onPlaybackStateChanged` | Play/pause/stop transition        | `{ state, position_secs }`            |
| `onLibraryScanned`     | Library scan completes              | `{ track_count }`                     |
| `customUIPanel`        | Plugin provides a UI panel          | `{ container: HTMLElement }`          |
| `audioTransform`       | Real-time audio processing          | `{ channelData, sampleRate }`         |

## Consequences

- Plugin authors write standard ES modules — no special toolchain.
- The Rust side stays focused on I/O, discovery, and state; no embedded JS engine.
- Adding WASM support later only requires a new `engine: "wasm"` manifest value
  and a WASM loader in the runtime — the manifest schema and hook contracts
  remain unchanged.
