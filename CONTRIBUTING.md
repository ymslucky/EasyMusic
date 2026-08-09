# Contributing to EasyMusic

First off, thanks for taking the time to contribute! 🎉

The following is a set of guidelines for contributing to EasyMusic. These are
mostly guidelines, not rules. Use your best judgment, and feel free to propose
changes to this document in a pull request.

## Code of Conduct

Be respectful and constructive. We expect all contributors to maintain a
welcoming and inclusive environment. Personal attacks, harassment, and
disruptive behavior will not be tolerated.

## How Can I Contribute?

### Reporting Bugs

1. Check the [issue tracker](https://github.com/ymslucky/EasyMusic/issues) to
   see if the bug has already been reported.
2. If not, open a new issue with:
   - A clear, descriptive title
   - Your OS and version (Windows / macOS / Linux distribution)
   - EasyMusic version (or the commit hash if building from source)
   - Steps to reproduce the issue
   - Expected behavior vs. actual behavior
   - Relevant logs or screenshots

### Suggesting Enhancements

1. Search the issue tracker for existing suggestions.
2. Open a new issue with the `enhancement` label, including:
   - A clear description of the proposed feature
   - The use case it addresses
   - Any alternative solutions you've considered

### Pull Requests

1. Fork the repository and create a feature branch from `main`:
   ```bash
   git checkout -b feat/my-feature
   ```
2. Keep your changes focused — one feature or bugfix per PR.
3. Follow the code style and conventions of the codebase.
4. Add or update tests for your changes (see [Testing](#testing)).
5. Ensure CI passes locally before pushing:
   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets -D warnings
   cargo test --all
   npm --prefix frontend run lint
   npm --prefix frontend run build
   ```
6. Write a clear commit message (see [Commit Convention](#commit-convention)).
7. Open a pull request and link any related issues.

## Development Setup

### Prerequisites

- [Node.js](https://nodejs.org/) ≥ 18 and npm
- [Rust](https://rustup.rs/) stable toolchain
- Platform-specific Tauri v2 system dependencies:
  - **Linux (Debian/Ubuntu):**
    ```bash
    sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev \
        libayatana-appindicator3-dev librsvg2-dev
    ```
  - **macOS:** Xcode Command Line Tools
    ```bash
    xcode-select --install
    ```
  - **Windows:** [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) + WebView2 runtime

### Getting Started

```bash
git clone https://github.com/ymslucky/EasyMusic.git
cd easymusic
npm install
npm --prefix frontend install
npm run tauri:dev
```

### Repository Layout

```
.
├── .github/workflows/   # CI + release pipelines
├── frontend/            # Next.js 16 frontend (static export)
├── src-tauri/           # Tauri app shell + Rust command layer
├── crates/
│   ├── easy-music-core/         # Core: library, playback, scanner, plugins
│   └── easy-music-plugin-sdk/   # Plugin SDK: manifest, permissions, hooks
├── plugins/             # Example plugins
└── docs/                # Architecture docs + ADRs
```

See [`docs/architecture.md`](docs/architecture.md) for a detailed overview.

## Code Style

### Rust

- Run `cargo fmt --all` before committing.
- `cargo clippy` must pass with zero warnings (`-D warnings` is enforced in CI).
- Prefer descriptive names; avoid single-letter variables except in short
  closures or loop indices.
- Keep public APIs documented with `///` doc comments.

### TypeScript / React (Frontend)

- Follow the existing ESLint configuration (`eslint-config-next`).
- Prefer function components with hooks.
- Use TypeScript strict types — avoid `any`.

## Testing

### Rust Tests

```bash
cargo test --all
```

Tests live alongside the source in each crate. Integration tests go in
`tests/` directories. The core crate includes end-to-end tests that scan and
index a temp directory of WAV files.

### Frontend Tests

Currently, the frontend relies on TypeScript type-checking and ESLint. If you
add user-facing components, consider adding component tests with a framework of
your choice (Vitest, React Testing Library).

## Commit Convention

We follow a lightweight [Conventional Commits](https://www.conventionalcommits.org/) style:

```
<type>(<scope>): <description>
```

Common types:

| Type       | Use for                                          |
|------------|--------------------------------------------------|
| `feat`     | A new feature                                    |
| `fix`      | A bug fix                                        |
| `docs`     | Documentation only changes                       |
| `style`    | Code style/formatting (no logic change)          |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test`     | Adding or correcting tests                       |
| `chore`    | Build process, tooling, dependencies             |
| `ci`       | CI configuration changes                         |

Example: `feat(playback): add gapless playback support`

## Plugin Contributions

Plugins are a first-class part of EasyMusic! If you'd like to add an example
plugin:

1. Create a subdirectory under `plugins/` with a `plugin.json` manifest.
2. See [`docs/plugin-development.md`](docs/plugin-development.md) for the full
   guide and the `plugins/lyrics-display/` example for reference.
3. Include a `README.md` in your plugin directory.

## Architecture Decision Records (ADRs)

Significant architectural decisions are documented in
[`docs/adr/`](docs/adr/). If your PR involves a notable architectural change,
consider adding a new ADR following the format of
[ADR-0001](docs/adr/0001-plugin-system.md).

## License

By contributing, you agree that your contributions will be licensed under the
[MIT License](LICENSE).
