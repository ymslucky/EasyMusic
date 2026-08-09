# plugins/

Placeholder for EasyMusic plugin packages.

Planned layout:

- `frontend/` — React plugins (UI panels, views) consumed by the Next.js app
- `backend/` — Rust plugin crates (audio backends, metadata scrapers) wired
  into `easy-music-core` and exposed via Tauri commands

Nothing to see here yet — this directory exists to keep the monorepo shape
stable while the core is being built.
