# EasyMusic 🎵

Cross-platform desktop music application.

**Stack:** [Tauri v2](https://v2.tauri.app/) · [Next.js](https://nextjs.org/) (static export / SSG) · [Rust](https://www.rust-lang.org/)

## Repository layout

```
.
├── frontend/            # Next.js frontend (static export → out/)
├── src-tauri/           # Tauri app: window shell + Rust command layer
│   ├── src/
│   │   ├── main.rs      # binary entry
│   │   ├── lib.rs       # Tauri builder + command registration
│   │   └── commands.rs  # #[tauri::command] wrappers over the core crate
│   ├── capabilities/    # Tauri v2 permission capabilities
│   └── tauri.conf.json  # app config (window, build hooks, bundling)
├── crates/
│   └── easy-music-core/ # Shared Rust core: library management, playback engine
└── plugins/             # (placeholder) frontend/backend plugin directory
```

## Prerequisites

- [Node.js](https://nodejs.org/) ≥ 18 and npm
- [Rust](https://rustup.rs/) stable toolchain
- Linux: `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev` (see Tauri docs for other platforms)

## Development

```bash
# 1. Install JS deps (root + frontend)
npm install
npm --prefix frontend install

# 2. Launch the app (Next dev server + Tauri window)
npm run tauri:dev
```

`npm run tauri:dev` starts `next dev` (port 1420) and opens the Tauri window
pointing at it. Hot-reloads both the frontend and the Rust command layer.

## Building

```bash
npm run tauri:build
```

`next build` produces the static export into `frontend/out/`, which Tauri
bundles into the platform installers (`.deb`/`.AppImage` on Linux, `.msi`/`.exe`
on Windows, `.dmg`/`.app` on macOS).

## Rust workspace

The Cargo workspace has two members:

| Crate | Role |
|---|---|
| `src-tauri` | Tauri app: window, `#[tauri::command]`s, plugin registration |
| `crates/easy-music-core` | Framework-agnostic core: `library` (LibraryManager), `playback` (PlaybackEngine), `error` |

Keep business logic in `easy-music-core`; `src-tauri` stays a thin shell.

## Status

Scaffold — blank window with the Next.js page renders; greet/library-stats
commands prove the frontend → Tauri → core-crate pipeline is wired.
