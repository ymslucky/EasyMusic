<div align="center">

# 🎵 EasyMusic

### Cross-platform desktop music player built with Tauri, Next.js, and Rust

[![CI](https://github.com/ymslucky/EasyMusic/actions/workflows/ci.yml/badge.svg)](https://github.com/ymslucky/EasyMusic/actions/workflows/ci.yml)
[![Release](https://github.com/ymslucky/EasyMusic/actions/workflows/release.yml/badge.svg)](https://github.com/ymslucky/EasyMusic/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-orange.svg)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-stable-dea584.svg)](https://www.rust-lang.org/)

A fast, lightweight, extensible music player for Windows, macOS, and Linux.
Scan your local music library, browse by track/album/artist, create playlists,
manage playback with a full transport engine — and extend everything with
JavaScript plugins.

</div>

---

## ✨ Features

- **Cross-platform** — Native desktop apps for Windows, macOS, and Linux from
  a single codebase (no Electron).
- **Library management** — Recursive directory scanner with automatic metadata
  extraction (ID3, Vorbis, FLAC, MP4 tags) backed by a fast SQLite database.
- **Rich browsing** — Browse tracks, albums, and artists. Full-text search
  across title, artist, album, and genre. Structured filters (artist, album,
  genre, duration range).
- **Playlists** — Full CRUD: create, rename, delete, add/remove tracks,
  reorder via drag-and-drop.
- **Playback engine** — Transport state machine with queue management,
  shuffle, repeat (off/all/one), seek, and volume control. Pluggable
  `AudioSink` trait for swappable audio backends.
- **Plugin system** — Extend the app with JavaScript plugins. Custom UI
  panels, track-change reactions, lyrics fetchers, audio visualizers,
  equalizers — all with a permission-based security model.
- **Virtualized UI** — Handles large libraries (10k+ tracks) smoothly with
  `@tanstack/react-virtual`.
- **Beautiful dark interface** — Modern React 19 + Tailwind CSS v4 design.

## Screenshots / UI Mockups

The app features five primary views accessible from a sidebar:

```
┌──────────────────────────────────────────────────────┐
│  EasyMusic                                    ─ □ ✕  │
├────────┬─────────────────────────────────────────────┤
│        │                                             │
│  🎵    │   Library                                   │
│  Library│   ┌──────────────────────────────────────┐  │
│  💿     │   │ ♪  Title        Artist   Album  Time │  │
│  Albums │   ├──────────────────────────────────────┤  │
│  🎤     │   │ 1  Track One    Artist A  …     3:42 │  │
│  Artists│   │ 2  Track Two    Artist B  …     4:15 │  │
│  ▶ List │   │ 3  Track Three  Artist A  …     2:58 │  │
│  ⚙ Set  │   │ 4  Track Four   Artist C  …     5:30 │  │
│        │   │ …                                    │  │
│        │   └──────────────────────────────────────┘  │
│        │                                             │
├────────┴─────────────────────────────────────────────┤
│  ◀◀  ▶  ▶▶   ━━━━━●━━━━━━━   🔀 🔁    🔊 ━━━●━━━   │
│  Now Playing: Track One — Artist A         1:23/3:42 │
└──────────────────────────────────────────────────────┘
```

### Views

| View       | Description                                                        |
|------------|--------------------------------------------------------------------|
| **Library**| Virtualized track table with sortable columns and live search     |
| **Albums** | Grid view of albums with cover art and track counts               |
| **Artists**| Artist list with album and track counts                           |
| **Playlists**| Create/manage playlists with drag-and-drop reordering           |
| **Settings**| Configure library paths, manage plugins, app preferences        |

## Tech Stack

| Layer          | Technology                                              |
|----------------|---------------------------------------------------------|
| **Desktop**    | [Tauri v2](https://v2.tauri.app/) — Rust + WebView      |
| **Frontend**   | [Next.js 16](https://nextjs.org/), React 19, TypeScript |
| **Styling**    | [Tailwind CSS v4](https://tailwindcss.com/)             |
| **State**      | [Zustand](https://github.com/pmndrs/zustand)            |
| **Backend**    | [Rust](https://www.rust-lang.org/) workspace            |
| **Database**   | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) (bundled) |
| **Metadata**   | [lofty](https://github.com/Serial-ATA/lofty-rs) (ID3/Vorbis/FLAC/MP4) |
| **Virtualization** | [@tanstack/react-virtual](https://tanstack.com/virtual) |
| **Build CI/CD**| GitHub Actions (4-platform release matrix)              |

## Prerequisites

- [Node.js](https://nodejs.org/) ≥ 18 and npm
- [Rust](https://rustup.rs/) stable toolchain
- Platform-specific Tauri v2 dependencies:
  - **Linux (Debian/Ubuntu):**
    ```bash
    sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
        libayatana-appindicator3-dev librsvg2-dev
    ```
  - **macOS:** `xcode-select --install`
  - **Windows:** [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) + WebView2 runtime

## Quick Start

```bash
# Clone
git clone https://github.com/ymslucky/EasyMusic.git
cd easymusic

# Install dependencies
npm install
npm --prefix frontend install

# Run in development (hot-reload frontend + Rust)
npm run tauri:dev
```

The dev server starts Next.js on `http://localhost:1420` and opens the Tauri
window automatically.

## Build Commands

| Command               | Description                                  |
|-----------------------|----------------------------------------------|
| `npm install`         | Install root dependencies (Tauri CLI)        |
| `npm --prefix frontend install` | Install frontend dependencies      |
| `npm run tauri:dev`   | Launch app in dev mode (hot-reload)          |
| `npm run tauri:build` | Build production installers for your platform|
| `npm run dev`         | Frontend-only dev server (no Tauri window)   |
| `npm run build`       | Frontend-only production build (static export)|

### Building Platform Installers

```bash
npm run tauri:build
```

This produces platform-native installers in `src-tauri/target/release/bundle/`:

| Platform | Output                              |
|----------|-------------------------------------|
| Linux    | `.deb`, `.AppImage`                |
| macOS    | `.dmg`, `.app`                     |
| Windows  | `.msi`, `.exe` (NSIS installer)    |

## Plugin Development

EasyMusic has a first-class JavaScript plugin system. Create a folder with a
`plugin.json` manifest and an `index.js` entry point, drop it in `plugins/`,
and restart the app.

### Quick Plugin Example

```
plugins/
└── my-plugin/
    ├── plugin.json
    └── index.js
```

**`plugin.json`:**
```json
{
  "id": "com.example.myplugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "entry": "index.js",
  "permissions": ["library:read"],
  "hooks": ["onTrackChanged"]
}
```

**`index.js`:**
```javascript
export default {
  onLoad(api) {
    api.log("Hello from my plugin!");
  },
  onTrackChanged(track) {
    console.log("Now playing:", track.title);
  },
};
```

📖 **Full guide:** [`docs/plugin-development.md`](docs/plugin-development.md)
📦 **Example plugin:** [`plugins/lyrics-display/`](plugins/lyrics-display/)
📐 **Architecture decision:** [`docs/adr/0001-plugin-system.md`](docs/adr/0001-plugin-system.md)

## Repository Layout

```
.
├── .github/workflows/           # CI + release pipelines
├── crates/
│   ├── easy-music-core/         # Core: library, playback, scanner, plugins
│   └── easy-music-plugin-sdk/   # Plugin SDK: manifest, permissions, hooks
├── docs/                        # Architecture docs + ADRs
├── frontend/                    # Next.js 16 frontend (static export)
├── plugins/                     # Example plugins
├── scripts/                     # Utility scripts (icon generation, etc.)
├── src-tauri/                   # Tauri app: window shell + Rust commands
│   ├── src/
│   │   ├── commands.rs          # #[tauri::command] wrappers
│   │   ├── plugin_commands.rs   # Plugin management commands
│   │   └── lib.rs               # Tauri builder + command registration
│   ├── capabilities/            # Tauri v2 permissions
│   ├── icons/                   # App icons (PNG/ICO/ICNS, all platforms)
│   └── tauri.conf.json          # App config (window, bundling, metadata)
├── Cargo.toml                   # Rust workspace root
├── LICENSE                      # MIT License
└── CONTRIBUTING.md              # Contribution guidelines
```

## Continuous Integration & Releases

### CI (`ci.yml`)

Runs on every PR and push to `main`:

- **Rust:** `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all`
- **Frontend:** `npm run lint`, `npm run build`

### Release (`release.yml`)

Triggers on tag pushes (`v*`) and produces installers for four targets:

| Platform         | Target   | Bundle formats      |
|-----------------|----------|---------------------|
| Ubuntu           | x86_64   | `.deb`, `.AppImage` |
| macOS (Intel)    | x86_64   | `.dmg`              |
| macOS (Apple Silicon) | aarch64 | `.dmg`         |
| Windows          | x86_64   | `.msi`, `.exe` (NSIS)|

Each release creates a **draft GitHub Release** with downloadable installers.

#### Release Runbook

Releases are triggered by pushing an annotated SemVer tag (`vX.Y.Z`). Before
tagging, bump the version in **both** `src-tauri/tauri.conf.json` and
`package.json` — the `validate-version` CI job will fail the release if the tag
doesn't match both files. The workflow then builds all desktop installers (and
the Android APK once `build-apk.yml` is merged to `main`), creates a **draft**
GitHub Release named `EasyMusic vX.Y.Z` with an auto-generated commit-log
changelog, and waits for the maintainer to review assets and click **Publish**.
Branch pushes, PRs, and manual `workflow_dispatch` runs produce throwaway
artifacts only — they never create or mutate a release. See the full
**[Release Policy](docs/release-policy.md)** for trigger rules, secret
requirements, and versioning conventions.

### Optional Code-Signing

Configure repository secrets for signed builds:

<details>
<summary>macOS signing</summary>

```
APPLE_CERTIFICATE
APPLE_CERTIFICATE_PASSWORD
APPLE_ID
APPLE_PASSWORD
APPLE_TEAM_ID
```

</details>

<details>
<summary>Windows / Tauri signing</summary>

```
TAURI_PRIVATE_KEY
TAURI_KEY_PASSWORD
```

</details>

## Documentation

- [Architecture Overview](docs/architecture.md) — System design with Mermaid diagrams
- [Plugin Development Guide](docs/plugin-development.md) — Complete plugin API reference
- [ADR-0001: Plugin System](docs/adr/0001-plugin-system.md) — Why we chose JS plugins
- [Release Policy](docs/release-policy.md) — Trigger rules, version source, changelog derivation
- [Contributing Guide](CONTRIBUTING.md) — How to contribute

## Supported Audio Formats

| Format  | Extension(s)              |
|---------|---------------------------|
| MP3     | `.mp3`                    |
| FLAC    | `.flac`                   |
| WAV     | `.wav`                    |
| Ogg     | `.ogg`                    |
| AAC/M4A | `.m4a`, `.aac`            |
| Opus    | `.opus`                   |
| WMA     | `.wma`                    |

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md)
for details on the development setup, code style, testing, and pull request
workflow.

### Quick CI Checklist

Before submitting a PR, make sure these pass locally:

```bash
cargo fmt --all --check
cargo clippy --all-targets -D warnings
cargo test --all
npm --prefix frontend run lint
npm --prefix frontend run build
```

## License

[MIT](LICENSE) © 2026 EasyMusic
