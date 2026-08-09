# Plugin Development Guide

This guide covers everything you need to create a plugin for EasyMusic. You'll
learn the plugin structure, manifest schema, hook system, permissions, and
work through a complete example.

> **Quick start:** Copy `plugins/lyrics-display/`, rename the fields in
> `plugin.json`, edit `index.js`, and restart EasyMusic.

## How Plugins Work

EasyMusic uses a **hybrid JavaScript plugin model**:

1. The **Rust host** discovers plugin directories, validates manifests, and
   manages enable/disable state.
2. The **frontend runtime** (running in the Tauri WebView) loads each enabled
   plugin's entry point as an ES module and dispatches lifecycle/event hooks.

Plugins are **plain JavaScript** — no build step, no special toolchain. Drop a
folder containing `plugin.json` + `index.js` into the `plugins/` directory and
restart the app.

## Plugin Structure

```
my-plugin/
├── plugin.json      ← Manifest (required)
├── index.js          ← Entry point (required)
└── README.md         ← Documentation (optional)
```

## The Manifest (`plugin.json`)

The manifest declares your plugin's identity, requested permissions, and
subscribed hooks.

```json
{
  "id": "com.yourname.yourplugin",
  "name": "Your Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "What your plugin does",
  "engine": "js",
  "entry": "index.js",
  "permissions": ["library:read", "ui:panel"],
  "hooks": ["onTrackChanged", "customUIPanel"],
  "min_app_version": "0.1.0"
}
```

### Manifest Fields

| Field             | Required | Description                                       |
|-------------------|----------|---------------------------------------------------|
| `id`              | Yes      | Globally unique reverse-DNS id (e.g. `com.example.lyrics`) |
| `name`            | Yes      | Human-readable display name                       |
| `version`         | Yes      | Semantic version (`MAJOR.MINOR.PATCH`)            |
| `author`          | Yes      | Author or organization name                       |
| `entry`           | Yes      | Entry-point file path (relative to plugin dir)    |
| `engine`          | No       | `"js"` (default) or `"wasm"` (future)            |
| `description`     | No       | One-line description                              |
| `permissions`     | No       | Array of capability strings                       |
| `hooks`           | No       | Array of event hook names                         |
| `min_app_version` | No       | Minimum EasyMusic version required               |

**Validation rules:**
- `id` must be non-empty and contain only `a-z A-Z 0-9 . - _`
- Unknown permissions and hooks are **rejected at parse time**
- Duplicate permissions and hooks are rejected

## Permissions

Plugins must declare every capability they need. The host rejects unknown
permissions, and the runtime enforces declared permissions before granting API
access.

| Permission         | Grants                                      |
|--------------------|---------------------------------------------|
| `library:read`     | Read track metadata via `api.getCurrentTrack()` |
| `playback:control` | Control playback via `api.playbackControl()`  |
| `network:fetch`    | Make HTTP requests via `api.proxiedFetch()`    |
| `ui:panel`         | Register a custom UI panel (`customUIPanel` hook) |
| `audio:transform`  | Real-time audio processing (`audioTransform` hook) |
| `playlist:access`  | Read and modify the playlist queue            |

## Hooks

Hooks are named lifecycle and event points. Your plugin subscribes to them in
the manifest and implements the corresponding methods in its default export.

| Hook                      | When fired                  | Method to implement        |
|---------------------------|-----------------------------|----------------------------|
| `onPluginLoaded`          | Plugin registered in runtime| `onLoad(api)`              |
| `onTrackChanged`          | Current track changes       | `onTrackChanged(track)`    |
| `onPlaybackStateChanged`  | Play/pause/stop transition  | `onPlaybackStateChanged(p)`|
| `onLibraryScanned`        | Library scan completes      | `onLibraryScanned(p)`      |
| `customUIPanel`           | Plugin renders a UI panel   | `customUIPanel(container)` |
| `audioTransform`          | Real-time audio processing  | `audioTransform(payload)`  |

## The Plugin Entry Point (`index.js`)

Write your plugin as an ES module that **default-exports** an object with hook
methods. Only implement the hooks you subscribed to.

```javascript
export default {
  onLoad(api) {
    api.log("Hello from my plugin!");
  },

  onTrackChanged(track) {
    console.log("Now playing:", track.title, "by", track.artist);
  },

  customUIPanel(container) {
    container.innerHTML = "<p>My custom panel</p>";
  },

  onUnload() {
    api.log("Goodbye!");
  },
};
```

## Plugin API Reference

The `onLoad(api)` hook receives a `PluginAPI` object. Methods are gated by
permissions — calling a method without the required permission is a no-op.

### `api.log(...args)`
Logs a message to the host console with the plugin id as prefix.

### `api.getCurrentTrack(): Track | null`
Returns the currently playing track, or `null`. **Requires:** `library:read`

```js
{
  id: "uuid",
  title: "Song Title",
  artist: "Artist Name",
  album: "Album Name",
  duration_secs: 240
}
```

### `api.proxiedFetch(url, opts?): Promise<string>`
Fetches a URL through the Rust host proxy (bypasses CORS, allows network
access). **Requires:** `network:fetch`

```js
const response = await api.proxiedFetch("https://api.example.com/lyrics", {
  method: "GET",
  headers: { "Accept": "application/json" }
});
```

### `api.playbackControl(action: "play" | "pause" | "stop")`
Dispatches a playback control command. **Requires:** `playback:control`

```js
api.playbackControl("pause");
```

## Complete Example: Now Playing Notifier

Here's a minimal plugin that logs track changes:

### `plugin.json`

```json
{
  "id": "com.example.nowplaying",
  "name": "Now Playing Logger",
  "version": "1.0.0",
  "author": "Jane Doe",
  "description": "Logs the currently playing track to the console.",
  "engine": "js",
  "entry": "index.js",
  "permissions": ["library:read"],
  "hooks": ["onTrackChanged", "onPluginLoaded"]
}
```

### `index.js`

```javascript
let _api = null;

export default {
  onLoad(api) {
    _api = api;
    api.log("Now Playing Logger started");
  },

  onTrackChanged(track) {
    const duration = formatDuration(track.duration_secs);
    _api.log(`♪ ${track.title} — ${track.artist} (${duration})`);
  },

  onUnload() {
    if (_api) _api.log("Now Playing Logger stopped");
    _api = null;
  },
};

function formatDuration(secs) {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${String(s).padStart(2, "0")}`;
}
```

## Advanced: Custom UI Panels

If your plugin declares `ui:panel` permission and the `customUIPanel` hook,
EasyMusic provides a container `HTMLElement` for you to render into. You can
use plain DOM manipulation:

```javascript
export default {
  customUIPanel(container) {
    container.innerHTML = `
      <div class="my-panel">
        <h3>My Plugin Panel</h3>
        <p>Content goes here.</p>
      </div>
    `;
  },
};
```

> **Security tip:** Always escape user-supplied text (track names, etc.) before
> inserting it as HTML to prevent injection. See the `escapeHtml` helper in the
> `lyrics-display` example plugin.

## Advanced: Network Fetching

To call external APIs (lyrics services, metadata enrichment, etc.):

1. Add `"network:fetch"` to your `permissions` array.
2. Use `api.proxiedFetch(url)` in your hook handlers.

```javascript
export default {
  onLoad(api) { this._api = api; },

  async onTrackChanged(track) {
    try {
      const url = `https://api.example.com/lyrics?artist=${encodeURIComponent(track.artist)}&title=${encodeURIComponent(track.title)}`;
      const response = await this._api.proxiedFetch(url);
      const data = JSON.parse(response);
      this._api.log(`Found lyrics: ${data.lyrics.substring(0, 50)}...`);
    } catch (err) {
      this._api.log("Failed to fetch lyrics:", err.message);
    }
  },
};
```

## Installing Plugins

### During Development

1. Create a subdirectory under `plugins/` in the EasyMusic project root.
2. Add `plugin.json` and `index.js`.
3. Restart EasyMusic, or use **Settings → Reload Plugins**.

### From a Path

Use the `install_plugin` Tauri command to install from an external directory:

```javascript
await invoke("install_plugin", { path: "/path/to/my-plugin" });
```

### Plugin Management Commands

| Tauri Command        | Action                             |
|----------------------|------------------------------------|
| `list_plugins`       | List all registered plugins        |
| `list_enabled_plugins` | List only enabled plugins        |
| `enable_plugin`      | Enable a plugin by id              |
| `disable_plugin`     | Disable a plugin by id             |
| `reload_plugins`     | Re-scan the plugins directory      |
| `install_plugin`     | Install from a path                |
| `uninstall_plugin`   | Remove a plugin by id              |
| `get_plugin_info`    | Get detailed info for one plugin   |
| `get_plugin_source`  | Read entry-point source (internal) |

## Security Model

- Plugins run in the same WebView origin as the app — this is
  defense-in-depth, not a hard sandbox.
- Permissions are validated at manifest parse time; unknown permissions are
  rejected.
- Network access is proxied through the Rust host, so plugins cannot make
  arbitrary requests without declaring `network:fetch`.
- A future version may introduce Web Workers or `<iframe sandbox>` for
  stronger isolation. See [ADR-0001](adr/0001-plugin-system.md) for the
  rationale.

## TypeScript Support

TypeScript definitions are available in
`frontend/src/lib/plugin-sdk/types.ts`. If you write your plugin in
TypeScript, you can reference these types:

```typescript
import type { EasyMusicPlugin, PluginAPI } from "../../frontend/src/lib/plugin-sdk/types";

const plugin: EasyMusicPlugin = {
  onLoad(api: PluginAPI) {
    api.log("Typed plugin loaded");
  },
};

export default plugin;
```

## Reference Implementation

The [`lyrics-display`](../plugins/lyrics-display/) plugin demonstrates:
- Manifest with permissions and hooks
- Lifecycle hooks (`onLoad`, `onUnload`)
- Event hook (`onTrackChanged`)
- Custom UI panel rendering (`customUIPanel`)
- HTML escaping for security

Use it as a starting template for your own plugins.
