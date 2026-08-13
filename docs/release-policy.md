# Release Policy

> **Version**: 1.0 — adopted 2026-08-14
> **Applies to**: EasyMusic desktop (`release.yml`) and Android (`build-apk.yml`, when merged to `main`) pipelines.

This document is the single source of truth for *how*, *when*, and *why*
EasyMusic releases are produced. It is intended to be enforceable by CI guards
and self-evident enough for a maintainer to follow without reading the workflow
source.

---

## 1. Trigger Mechanism — Tag Push

Releases are triggered **exclusively by pushing an annotated git tag** matching
the SemVer pattern:

```
v<major>.<minor>.<patch>          e.g.  v0.1.2
v<major>.<minor>.<patch>-<pre>    e.g.  v0.2.0-beta.1
```

Regex enforced by the `validate-version` job in `release.yml`:

```
^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$
```

| Trigger                           | What happens                                   | Release created? |
|-----------------------------------|------------------------------------------------|------------------|
| Push tag `v*`                     | Full build matrix + **draft** GitHub Release    | **Yes (draft)**   |
| Push to `main` (no tag)           | Build artifacts uploaded (CI sanity check)      | No               |
| Pull request                      | Build artifacts uploaded (integration test)     | No               |
| `workflow_dispatch` (manual)      | Build only — no release unless tag is checked out | No (default)    |

Branch pushes and PRs produce throwaway artifacts for testing; they **never**
create or mutate a GitHub Release.

### `workflow_dispatch` escape hatch

The `release.yml` workflow accepts a manual `workflow_dispatch` trigger for
emergency re-builds or debugging. Manual runs produce artifacts only; they do
not create a release unless triggered on a checked-out tag ref.

---

## 2. Version Source of Truth

**`src-tauri/tauri.conf.json` → `"version"`** is the canonical version string
for the entire project.

Two files must carry the identical version and are checked by CI:

| File                        | Field      |
|-----------------------------|------------|
| `src-tauri/tauri.conf.json` | `version`  |
| `package.json`              | `version`  |

The `validate-version` job fails the release if:

1. The tag (minus the `v` prefix) does not match `tauri.conf.json`.
2. The tag does not match `package.json`.
3. The tag is not valid SemVer.

### Release Procedure (maintainer runbook)

1. **Bump the version** in both `tauri.conf.json` and `package.json` on `main`.
2. **Commit** the version bump with a message like `chore(release): vX.Y.Z`.
3. **Create and push** the tag:
   ```bash
   git tag -a vX.Y.Z -m "EasyMusic vX.Y.Z"
   git push origin vX.Y.Z
   ```
4. **Wait** for CI. The `validate-version` → `build` pipeline runs; a draft
   GitHub Release is created with auto-generated changelog notes.
5. **Review** the draft release: edit the body, verify all assets (desktop
   installers + Android APK) are present.
6. **Publish** the draft release when satisfied.

> **Pre-release tags** (`-beta`, `-rc`, etc.) follow the same flow. The release
> is created with `prerelease: false` by default — the maintainer marks it as a
> pre-release in the GitHub UI if desired.

---

## 3. Draft vs. Immediate Publish

**All releases are created as drafts.** No release is published automatically.

- `releaseDraft: true` in `tauri-action` inputs (desktop).
- The `Publish to GitHub Release` step in `build-apk.yml` (Android) also uploads
  to the existing draft.
- The maintainer clicks **Publish** in the GitHub Releases UI after reviewing
  assets and release notes.

Rationale: APKs are currently debug-signed (production keystore not yet
configured). Draft-first gives a final safety net before anything is
publicly downloadable.

---

## 4. Release Name and Body Derivation

### Name

```
EasyMusic <tag>
```

Example: `EasyMusic v0.1.2`.

Implemented via `releaseName: 'EasyMusic ${{ github.ref_name }}'`.

### Body (changelog)

Auto-generated from the **commit log between the previous tag and the current
tag** at release time:

```
git log --pretty=format:"- %s" <previous-tag>..HEAD
```

If no previous tag exists, the 30 most recent commits are used.

The body is computed in the `validate-version` job (step `Generate changelog`)
and passed to `tauri-action` via `releaseBody`. The maintainer should review and
edit the draft release body before publishing — particularly to add a human
summary, breaking-change callouts, and known issues.

> A `CHANGELOG.md` file is **optional**. If one is added in the future, the
> maintainer should paste the relevant section into the draft release body
> before publishing; the auto-generated commit list serves as a fallback.

---

## 5. Android APK Releases

The `build-apk.yml` workflow (currently on feature branch `wt/t_750a16f4`,
pending merge to `main`) follows the same tag-triggered policy:

- Tag push `v*` → build APK + upload to the **same draft release** created by
  `release.yml`.
- APK filename: `EasyMusic-{tag}{-debug}.apk`.
- If `SIGNING_KEY` secret is absent, the APK is debug-signed and the filename
  carries a `-debug` suffix. Production releases require the four signing
  secrets to be configured first.

See task t_5a60091b audit report for the full APK pipeline details.

---

## 6. Secrets Required for Full Releases

| Secret                          | Purpose                    | Required for        |
|---------------------------------|----------------------------|----------------------|
| `SIGNING_KEY`                   | Android keystore (base64)  | Signed APK           |
| `KEYSTORE_PASSWORD`             | Keystore password          | Signed APK           |
| `KEY_ALIAS`                     | Key alias                  | Signed APK           |
| `KEY_PASSWORD`                  | Key password               | Signed APK           |
| `APPLE_CERTIFICATE`            | macOS codesign             | Signed macOS DMG     |
| `APPLE_CERTIFICATE_PASSWORD`   | Cert password              | Signed macOS DMG     |
| `KEYCHAIN_PASSWORD`            | CI keychain                | Signed macOS DMG     |
| `APPLE_ID` / `APPLE_PASSWORD`  | Notarization               | Notarized macOS DMG  |
| `APPLE_TEAM_ID`                | Notarization               | Notarized macOS DMG  |
| `TAURI_PRIVATE_KEY`            | Windows updater signing    | Signed Windows build |
| `TAURI_KEY_PASSWORD`           | Updater key password       | Signed Windows build |

Until signing secrets are configured, builds succeed but produce unsigned or
debug-signed artifacts. These are fine for testing but should not be published
as stable releases.

---

## 7. Versioning Conventions

We follow **Semantic Versioning** (`semver.org`):

| Bump type | When to use                                      |
|-----------|--------------------------------------------------|
| **patch** | Bug fixes, CI/tooling, no new features           |
| **minor** | New features, backward-compatible                 |
| **major** | Breaking changes (UI overhaul, data format, etc.) |

Current version lineage: `v0.1.0` → `v0.1.1`.
