#!/usr/bin/env bash
# =============================================================================
# EasyMusic — headless Tauri runtime launcher (rootless container / CI)
# -----------------------------------------------------------------------------
# The Tauri GUI needs the webkit2gtk-4.1 *runtime* (helper processes) at the
# system paths `/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/`. On a normal desktop
# those ship with `libwebkit2gtk-4.1-0`; in this rootless container they are
# only available inside the vendored sysroot (/opt/data/tauri-sysroot/root),
# so we expose them with `proot -b` bind mounts (no root required).
#
# Also required for the web process to come up:
#   * EGL/GLVND vendor config  -> __EGL_VENDOR_LIBRARY_DIRS (mesa swrast)
#   * compiled GSettings schema -> gschemas.compiled (see compile step below)
#   * software GL               -> LIBGL_ALWAYS_SOFTWARE=1 + GALLIUM_DRIVER
#
# Usage:
#   scripts/run-tauri-headless.sh            # run built debug binary headless
#   DISPLAY=:99 scripts/run-tauri-headless.sh  # attach to an existing Xvfb
#
# Environment overrides (all optional):
#   EASY_MUSIC_BIN   path to the easymusic binary (default: target/debug/easymusic)
#   PROOT_BIN        path to proot (default: /tmp/proot-x/usr/bin/proot)
#   TAURI_SYSROOT    path to the vendored sysroot (default: /opt/data/tauri-sysroot/root)
# =============================================================================
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EASY_MUSIC_BIN="${EASY_MUSIC_BIN:-$REPO_ROOT/target/debug/easymusic}"
PROOT_BIN="${PROOT_BIN:-/tmp/proot-x/usr/bin/proot}"
SYSROOT="${TAURI_SYSROOT:-/opt/data/tauri-sysroot/root}"
TALLOC_LIB="${TALLOC_LIB:-/tmp/talloc-x/usr/lib/x86_64-linux-gnu}"

if [[ ! -x "$EASY_MUSIC_BIN" ]]; then
  echo "error: binary not found at $EASY_MUSIC_BIN — build first (cargo build --workspace)" >&2
  exit 1
fi
if [[ ! -x "$PROOT_BIN" ]]; then
  echo "error: proot not found at $PROOT_BIN (needed for rootless bind mounts)" >&2
  exit 1
fi
if [[ ! -d "$SYSROOT" ]]; then
  echo "error: sysroot not found at $SYSROOT (vendored webkit2gtk runtime)" >&2
  exit 1
fi

# --- 0. Ensure GSettings schemas are compiled (GLib hard-fails without it) ---
SCHEMA_DIR="$SYSROOT/usr/share/glib-2.0/schemas"
COMPILER="$SYSROOT/usr/lib/x86_64-linux-gnu/glib-2.0/glib-compile-schemas"
if [[ -x "$COMPILER" && ! -f "$SCHEMA_DIR/gschemas.compiled" ]]; then
  echo "[run-tauri] compiling GSettings schemas..."
  "$COMPILER" "$SCHEMA_DIR"
fi

# --- 1. WebKitWebProcess wrapper -------------------------------------------
# proot has trouble tracing the raw WebKitWebProcess ELF when it is spawned
# directly; exec'ing it through a tiny shell wrapper is proven to work.
WEBPROC_WRAPPER="${WEBPROC_WRAPPER:-/tmp/easymusic-webproc-wrapper.sh}"
REAL_WEBPROC="$SYSROOT/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitWebProcess"
cat > "$WEBPROC_WRAPPER" <<EOF
#!/bin/bash
exec "$REAL_WEBPROC" "\$@"
EOF
chmod +x "$WEBPROC_WRAPPER"

# --- 2. Environment ---------------------------------------------------------
export LD_LIBRARY_PATH="$SYSROOT/usr/lib/x86_64-linux-gnu:$SYSROOT/usr/lib:${TALLOC_LIB}:${LD_LIBRARY_PATH:-}"
export DISPLAY="${DISPLAY:-:99}"
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1
export LIBGL_ALWAYS_SOFTWARE=1
export GALLIUM_DRIVER=llvmpipe
export __EGL_VENDOR_LIBRARY_DIRS="$SYSROOT/usr/share/glvnd/egl_vendor.d"
export __EGL_VENDOR_LIBRARY_FILENAMES="$SYSROOT/usr/share/glvnd/egl_vendor.d/50_mesa.json"

# --- 3. proot bind mounts + exec -------------------------------------------
exec "$PROOT_BIN" \
  -b "$WEBPROC_WRAPPER:/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitWebProcess" \
  -b "$SYSROOT/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess:/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess" \
  -b "$SYSROOT/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitGPUProcess:/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitGPUProcess" \
  -b "$SYSROOT/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/injected-bundle:/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/injected-bundle" \
  -b "$SYSROOT/usr/lib/x86_64-linux-gnu/dri:/usr/lib/x86_64-linux-gnu/dri" \
  -b "$SCHEMA_DIR:/usr/share/glib-2.0/schemas" \
  -b "$SYSROOT/usr/share/glvnd:/usr/share/glvnd" \
  -b "$SYSROOT/usr/lib/x86_64-linux-gnu/gio/modules:/usr/lib/x86_64-linux-gnu/gio/modules" \
  "$EASY_MUSIC_BIN"
