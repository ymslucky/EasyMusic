# plugins/

EasyMusic plugin packages. Each subdirectory containing a `plugin.json`
manifest is treated as one plugin.

## Layout

```
plugins/
└── lyrics-display/
    ├── plugin.json      ← manifest (required)
    ├── index.js          ← entry point (required, matches "entry" field)
    └── README.md         ← documentation (optional)
```

## Creating a Plugin

1. Create a directory under `plugins/` for your plugin.
2. Add a `plugin.json` manifest:

```json
{
  "id": "com.yourname.yourplugin",
  "name": "Your Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "What it does",
  "engine": "js",
  "entry": "index.js",
  "permissions": ["library:read", "ui:panel"],
  "hooks": ["onTrackChanged", "customUIPanel"]
}
```

3. Write the entry point (`index.js`) as an ES module that default-exports
   a plugin object:

```javascript
export default {
  onLoad(api) {
    api.log("Hello from my plugin!");
  },
  onTrackChanged(track) {
    console.log("Now playing:", track.title);
  },
  customUIPanel(container) {
    container.innerHTML = "<p>My custom panel</p>";
  },
};
```

4. Restart EasyMusic (or use the "Reload Plugins" button in Settings).

## Manifest Fields

| Field            | Required | Description                              |
|------------------|----------|------------------------------------------|
| `id`             | Yes      | Unique reverse-DNS id                    |
| `name`           | Yes      | Display name                             |
| `version`        | Yes      | Semantic version                         |
| `author`         | Yes      | Author name                              |
| `entry`          | Yes      | Entry-point file (relative)              |
| `engine`         | No       | `"js"` (default) or `"wasm"` (future)   |
| `description`    | No       | One-line description                     |
| `permissions`    | No       | Capabilities requested                   |
| `hooks`          | No       | Event hooks subscribed                   |
| `min_app_version`| No       | Minimum app version required             |

## Permissions

| Permission         | Grants                              |
|--------------------|-------------------------------------|
| `library:read`     | Read track metadata                 |
| `playback:control` | Control playback (play/pause/stop)  |
| `network:fetch`    | Make proxied HTTP requests          |
| `ui:panel`         | Register a custom UI panel          |
| `audio:transform`  | Apply real-time audio effects       |
| `playlist:access`  | Read/modify playlists              |

## Hooks

| Hook                      | Triggered when              |
|---------------------------|-----------------------------|
| `onPluginLoaded`          | Plugin loaded               |
| `onTrackChanged`          | Current track changes       |
| `onPlaybackStateChanged`  | Play/pause/stop             |
| `onLibraryScanned`        | Library scan completes      |
| `customUIPanel`           | Render a UI panel           |
| `audioTransform`          | Real-time audio processing  |

## Included Plugins

- **lyrics-display** — Example plugin showing lyrics for the current track.
