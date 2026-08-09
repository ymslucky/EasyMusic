# EasyMusic Architecture

This document describes the high-level architecture of EasyMusic, the major
components, data flow, and key design decisions.

## Overview

EasyMusic is a cross-platform desktop music player built with
[Tauri v2](https://v2.tauri.app/), [Next.js](https://nextjs.org/), and
[Rust](https://www.rust-lang.org/). It is organized as a Cargo workspace with
a Next.js frontend that Tauri bundles into a native desktop application.

```mermaid
graph TB
    subgraph Frontend["Frontend — Next.js / React"]
        UI[App Shell & Views]
        Store[Zustand Store]
        PluginRT[Plugin Runtime JS]
    end

    subgraph Tauri["Tauri Shell"]
        Cmds["#[tauri::command] Layer"]
    end

    subgraph Core["easy-music-core"]
        Library["LibraryManager"]
        Playback["PlaybackEngine"]
        Scanner["Scanner / Tag Reader"]
        PluginMgr["PluginManager"]
    end

    subgraph SDK["easy-music-plugin-sdk"]
        Manifest["PluginManifest"]
        Perms["Permissions"]
        Hooks["PluginHook"]
    end

    subgraph Storage["Local Storage"]
        SQLite[("SQLite DB")]
        FS["Audio Files on Disk"]
    end

    UI -->|"invoke()"| Cmds
    PluginRT -->|"invoke()"| Cmds
    Cmds --> Library
    Cmds --> Playback
    Cmds --> PluginMgr
    Library --> SQLite
    Scanner --> FS
    Library --> Scanner
    PluginMgr --> SDK
```

## Technology Stack

| Layer         | Technology                                        |
|---------------|---------------------------------------------------|
| Desktop shell | Tauri v2 (Rust, WebView-based, no Electron)       |
| Frontend      | Next.js 16, React 19, TypeScript, Tailwind CSS v4 |
| State         | Zustand                                           |
| Virtualization| @tanstack/react-virtual                           |
| Backend logic | Rust (workspace crates)                           |
| Database      | SQLite (via `rusqlite`, bundled)                  |
| Metadata      | `lofty` (ID3, Vorbis, FLAC, MP4, etc.)            |
| Plugin runtime| JavaScript (in WebView, ES module based)          |

## Crate Structure

### Cargo Workspace

```
src-tauri/                        # Tauri app (binary)
crates/easy-music-core/           # Framework-agnostic core
crates/easy-music-plugin-sdk/     # Plugin SDK (shared types)
```

### easy-music-core

The framework-agnostic heart of the application. Contains all business logic
and is independently testable without Tauri or a GUI.

| Module        | Responsibility                                                        |
|---------------|-----------------------------------------------------------------------|
| `library`     | `LibraryManager` — SQLite-backed track/album/artist/playlist CRUD     |
| `scanner`     | Recursive directory scanner; reads metadata via `lofty`              |
| `playback`    | `PlaybackEngine` — transport state machine, queue, shuffle/repeat     |
| `plugins`     | `PluginManager` — discovers, validates, enables/disables plugins      |
| `db`          | SQLite connection + schema bootstrap                                  |
| `models`      | Serde-serializable data types (`Track`, `Album`, `Playlist`, …)       |
| `error`       | `CoreError` enum + `CoreResult` alias                                 |

### easy-music-plugin-sdk

A dependency-light crate shared between the plugin host and plugin authors.
Contains the manifest schema, permission model, and hook catalog. Parsing is
strict — unknown permissions or hooks are rejected at deserialization time.

### src-tauri

The thin Tauri shell. `#[tauri::command]` functions wrap `easy-music-core`
methods so the frontend can call them via `invoke()`. No business logic lives
here — it is purely a bridge.

## Data Flow

### Library Scan

```mermaid
sequenceDiagram
    participant U as User
    participant FE as Frontend
    participant CMD as library_scan()
    participant SC as Scanner
    participant DB as SQLite

    U->>FE: Clicks "Scan Folder"
    FE->>CMD: invoke("library_scan", root)
    CMD->>SC: scan_directory(root)
    SC->>SC: Walk directory tree
    SC->>SC: Parse metadata (lofty)
    SC-->>CMD: Vec<Track>
    CMD->>DB: Bulk upsert tracks/albums/artists
    DB-->>CMD: ScanResult
    CMD-->>FE: { scanned, added, errors }
    FE->>U: Updated library view
```

### Playback

```mermaid
sequenceDiagram
    participant FE as Frontend
    participant CMD as playback_play_track()
    participant PE as PlaybackEngine
    participant SINK as AudioSink

    FE->>CMD: invoke("playback_play_track", track)
    CMD->>PE: engine.play(track)
    PE->>SINK: sink.play(track, 0)
    PE->>PE: Set state = Playing
    PE-->>CMD: PlaybackStatus
    CMD-->>FE: { state, track, position, ... }
```

### Plugin Lifecycle

```mermaid
sequenceDiagram
    participant FE as Frontend Runtime
    participant CMD as Tauri Commands
    participant PM as PluginManager
    participant FS as Filesystem

    Note over PM,FS: App startup
    PM->>FS: Scan plugins/ directory
    FS-->>PM: Subdirectories with plugin.json
    PM->>PM: Parse + validate manifests
    PM->>PM: Register + set status

    Note over FE,CMD: Frontend init
    FE->>CMD: invoke("list_enabled_plugins")
    CMD->>PM: manager.enabled()
    PM-->>CMD: Vec<PluginInfo>
    CMD-->>FE: Plugin list

    loop For each enabled plugin
        FE->>CMD: invoke("get_plugin_source", id)
        CMD->>PM: plugin.read_entry_source()
        PM-->>FE: JS source code
        FE->>FE: Load ES module
        FE->>FE: Call onLoad(api)
    end
```

## Database Schema

The library is backed by a SQLite database with the following normalized
schema:

```mermaid
erDiagram
    artists ||--o{ albums : "1:N"
    artists ||--o{ tracks : "1:N"
    albums  ||--o{ tracks : "1:N"
    playlists ||--o{ playlist_tracks : "1:N"
    tracks    ||--o{ playlist_tracks : "1:N"

    artists {
        TEXT id PK
        TEXT name UK
    }
    albums {
        TEXT id PK
        TEXT title
        TEXT artist_id FK
        INTEGER year
        TEXT genre
    }
    tracks {
        TEXT id PK
        TEXT title
        TEXT artist_id FK
        TEXT album_id FK
        TEXT genre
        TEXT path UK
        INTEGER duration_secs
        INTEGER track_number
        TEXT file_format
    }
    playlists {
        TEXT id PK
        TEXT name
        TEXT created_at
    }
    playlist_tracks {
        TEXT playlist_id PK
        TEXT track_id PK
        INTEGER position
    }
```

## Supported Audio Formats

The scanner recognizes and parses metadata from:

| Format  | Extension(s)                |
|---------|-----------------------------|
| MP3     | `.mp3`                      |
| FLAC    | `.flac`                     |
| WAV     | `.wav`                      |
| Ogg     | `.ogg`                      |
| AAC/M4A | `.m4a`, `.aac`              |
| Opus    | `.opus`                     |
| WMA     | `.wma`                      |

## Tauri Commands

The frontend communicates with the backend exclusively through Tauri
`invoke()` calls. Key command groups:

| Group     | Commands                                                     |
|-----------|--------------------------------------------------------------|
| Library   | `library_open_db`, `library_scan`, `library_metadata`       |
| Tracks    | `tracks_all`, `track_get`, `tracks_search`, `tracks_filter` |
| Albums    | `albums_all`, `artists_all`                                  |
| Playlists | `playlist_create`, `playlist_rename`, `playlist_delete`, …  |
| Playback  | `playback_play_track`, `playback_pause`, `playback_resume`, …|
| Plugins   | `list_plugins`, `enable_plugin`, `get_plugin_source`, …     |

See `src-tauri/src/commands.rs` and `src-tauri/src/plugin_commands.rs` for the
complete list.

## CI/CD Pipeline

```mermaid
graph LR
    Push["Push / PR"] --> CI["ci.yml"]
    Tag["v* tag"] --> Release["release.yml"]

    CI --> Fmt["cargo fmt --check"]
    CI --> Clippy["cargo clippy -D warnings"]
    CI --> Test["cargo test"]
    CI --> Lint["npm run lint"]
    CI --> Build["npm run build"]

    Release --> Linux["Linux deb/AppImage"]
    Release --> MacIntel["macOS Intel .dmg"]
    Release --> MacARM["macOS ARM .dmg"]
    Release --> Windows["Windows .msi/.exe"]

    Linux --> Draft["Draft GitHub Release"]
    MacIntel --> Draft
    MacARM --> Draft
    Windows --> Draft
```

## Further Reading

- [ADR-0001: Plugin System Architecture](adr/0001-plugin-system.md)
- [Plugin Development Guide](plugin-development.md)
- [Contributing Guide](../CONTRIBUTING.md)
