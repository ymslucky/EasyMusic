# Lyrics Display Plugin

An example EasyMusic plugin that demonstrates the plugin lifecycle:
- `onLoad()` — receive the API context
- `onTrackChanged()` — react to track changes
- `customUIPanel()` — render a custom UI panel with lyrics

## Manifest

```json
{
  "id": "com.easymusic.example.lyrics",
  "permissions": ["library:read", "ui:panel"],
  "hooks": ["onTrackChanged", "customUIPanel"]
}
```

## How it works

1. When a track changes, the plugin stores the current track info.
2. The `customUIPanel` hook renders a lyrics panel in the app.
3. The panel displays the track name and sample lyrics lines.

## Extending

To fetch real lyrics, add `"network:fetch"` to permissions and use
`api.proxiedFetch()` to call a lyrics API in `onTrackChanged`.
