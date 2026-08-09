/**
 * EasyMusic Example Plugin — Lyrics Display
 *
 * Demonstrates the plugin lifecycle:
 * 1. onLoad() — receive the API context
 * 2. onTrackChanged() — react to track changes
 * 3. customUIPanel() — render a custom UI panel
 *
 * This plugin shows a lyrics area below the player. Since we don't have
 * a real lyrics API, it displays placeholder synced lyrics.
 */

/** Plugin state */
let _api = null;
let _currentTrack = null;
let _panelEl = null;

/**
 * Sample lyrics lines with timestamps (in seconds).
 * In a real plugin, these would be fetched from an API (requires network:fetch).
 */
const SAMPLE_LYRICS = [
  { time: 0, text: "♪ ♪ ♪" },
  { time: 5, text: "Welcome to EasyMusic" },
  { time: 12, text: "This is the lyrics plugin" },
  { time: 20, text: "Playing your favorite tune" },
  { time: 30, text: "La la la, oh oh oh" },
  { time: 45, text: "Music fills the air tonight" },
  { time: 60, text: "♪ ♪ ♪" },
];

/** Default export — the plugin implementation. */
export default {
  /**
   * Called once when the plugin is loaded.
   * @param {import("../../frontend/src/lib/plugin-sdk/types").PluginAPI} api
   */
  onLoad(api) {
    _api = api;
    api.log("Lyrics Display plugin loaded");

    // Try to get the current track if one is playing
    if (api.getCurrentTrack) {
      _currentTrack = api.getCurrentTrack();
    }
  },

  /**
   * Called when the current track changes.
   * @param {import("../../frontend/src/lib/plugin-sdk/types").Track} track
   */
  onTrackChanged(track) {
    _currentTrack = track;
    if (_api) {
      _api.log(`Track changed: ${track.title} by ${track.artist}`);
    }
    updateLyricsDisplay();
  },

  /**
   * Called to render the custom UI panel.
   * @param {HTMLElement} container
   */
  customUIPanel(container) {
    _panelEl = container;

    container.innerHTML = `
      <div style="
        padding: 12px;
        background: rgba(99, 102, 241, 0.05);
        border: 1px solid rgba(99, 102, 241, 0.2);
        border-radius: 12px;
        min-height: 80px;
      ">
        <div style="
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 0.1em;
          color: #818CF8;
          margin-bottom: 8px;
          font-weight: 600;
        ">
          🎵 Lyrics — Lyrics Display Plugin
        </div>
        <div id="easymusic-lyrics-content" style="
          font-size: 14px;
          line-height: 1.6;
          color: #94A3B8;
        ">
          No track playing.
        </div>
      </div>
    `;

    updateLyricsDisplay();
  },

  /** Called when the plugin is unloaded. */
  onUnload() {
    if (_api) {
      _api.log("Lyrics Display plugin unloading");
    }
    _panelEl = null;
    _currentTrack = null;
    _api = null;
  },
};

/** Update the lyrics display based on the current track. */
function updateLyricsDisplay() {
  if (!_panelEl) return;

  const contentEl = _panelEl.querySelector("#easymusic-lyrics-content");
  if (!contentEl) return;

  if (!_currentTrack) {
    contentEl.textContent = "No track playing.";
    contentEl.style.color = "#6B7280";
    return;
  }

  // Show track info and first few sample lyrics lines
  const lines = SAMPLE_LYRICS.slice(0, 4);
  const lyricsHTML = lines
    .map(
      (line, i) =>
        `<div style="margin: 4px 0; ${i === 1 ? "color: #6366F1; font-weight: 500;" : ""}">
          ${line.text}
        </div>`
    )
    .join("");

  contentEl.innerHTML = `
    <div style="margin-bottom: 8px; color: #E2E8F0; font-weight: 500;">
      ${escapeHtml(_currentTrack.title)} — ${escapeHtml(_currentTrack.artist)}
    </div>
    ${lyricsHTML}
  `;
}

/** Escape HTML to prevent injection from track metadata. */
function escapeHtml(str) {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}
