# EasyMusic 🎵

Cross-platform desktop music application.

[![CI](https://github.com/easymusic/easymusic/actions/workflows/ci.yml/badge.svg)](https://github.com/easymusic/easymusic/actions/workflows/ci.yml)
[![Release](https://github.com/easymusic/easymusic/actions/workflows/release.yml/badge.svg)](https://github.com/easymusic/easymusic/actions/workflows/release.yml)

**Stack:** [Tauri v2](https://v2.tauri.app/) · [Next.js](https://nextjs.org/) (static export / SSG) · [Rust](https://www.rust-lang.org/)

## Repository layout

```
.
├── .github/workflows/  # CI + release pipelines
├── frontend/            # Next.js frontend (static export → out/)
├── src-tauri/           # Tauri app: window shell + Rust command layer
│   ├── src/
│   │   ├── main.rs      # binary entry
│   │   ├── lib.rs       # Tauri builder + command registration
│   │   ├── commands.rs  # #[tauri::command] wrappers over the core crate
│   │   └── plugin_commands.rs
│   ├── capabilities/    # Tauri v2 permission capabilities
│   ├── icons/           # App icons (PNG/ICO/ICNS for all platforms)
│   └── tauri.conf.json  # app config (window, build hooks, bundling)
├── crates/
│   ├── easy-music-core/     # Core: library, playback, scanner, plugins
│   └── easy-music-plugin-sdk/  # Plugin SDK: manifest, permissions, hooks
└── plugins/             # Example plugins
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
Run it from the repository root (the `beforeDevCommand` resolves `frontend/`
relative to the invocation directory).

### Headless / CI (no display server)

```bash
# rootless GTK/WebKit sysroot + Xvfb (see the scaffold notes in the task log)
source /opt/data/env-tauri.sh
export DISPLAY=:99
Xvfb :99 -screen 0 1280x800x24 &
eval "$(dbus-launch --sh-syntax)"
export WEBKIT_DISABLE_DMABUF_RENDERER=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 \
       WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 LIBGL_ALWAYS_SOFTWARE=1
npm run tauri:dev
```

## Building

```bash
npm run tauri:build
```

`next build` produces the static export into `frontend/out/`, which Tauri
bundles into the platform installers (`.deb`/`.AppImage` on Linux, `.msi`/`.exe`
on Windows, `.dmg`/`.app` on macOS).

## Continuous Integration & Releases

### CI (`ci.yml`)

Runs on every PR and push to `main`:

- **Rust job**: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`
- **Frontend job**: `npm run lint`, `npm run build`

### Release (`release.yml`)

Triggers on tag pushes (`v*`) and pushes to `main`:

| Platform | Targets | Bundle formats |
|---|---|---|
| Ubuntu | x86_64 | `.deb`, `.AppImage` |
| macOS (Intel) | x86_64 | `.dmg` |
| macOS (ARM) | aarch64 | `.dmg` |
| Windows | x86_64 | `.msi`, `.exe` (NSIS) |

Each build uploads platform artifacts and creates a **draft GitHub Release** with
downloadable installers when a `v*` tag is pushed.

Optional code-signing (configure repo secrets when ready):
- **macOS**: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`
- **Windows**: `TAURI_PRIVATE_KEY`, `TAURI_KEY_PASSWORD`

## Rust workspace

The Cargo workspace has three members:

| Crate | Role |
|---|---|
| `src-tauri` | Tauri app: window, `#[tauri::command]`s, plugin registration |
| `crates/easy-music-core` | Framework-agnostic core: `library` (LibraryManager), `playback` (PlaybackEngine), `plugins` (PluginManager), `scanner` |
| `crates/easy-music-plugin-sdk` | Plugin SDK: manifest types, permissions, hooks (shared by host + plugin authors) |

Keep business logic in `easy-music-core`; `src-tauri` stays a thin shell.

## Status

Cross-platform desktop music app with library management, playback engine,
plugin system, and full CI/CD pipeline. CI passes on all three desktop
platforms; tagged pushes produce downloadable installers via GitHub Releases.
