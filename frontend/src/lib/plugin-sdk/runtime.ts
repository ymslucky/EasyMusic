/**
 * Plugin runtime — loads plugin scripts, dispatches hooks, manages lifecycle.
 *
 * This runs inside the Tauri WebView. It communicates with the Rust host
 * via Tauri commands to get the plugin list and entry-point source.
 */

import { isTauri } from "@tauri-apps/api/core";

import type {
  EasyMusicPlugin,
  Permission,
  PluginAPI,
  PluginHookName,
  PluginInfo,
  PluginManifest,
  Track,
  PlaybackStatePayload,
  LibraryScannedPayload,
} from "./types";

interface LoadedPlugin {
  info: PluginInfo;
  instance: EasyMusicPlugin | null;
  api: PluginAPI | null;
  source: string;
  error: string | null;
}

/**
 * Central plugin runtime. Manages plugin lifecycle, hook dispatch, and
 * acts as the bridge between the host (Tauri/Rust) and plugin instances.
 */
export class PluginRuntime {
  private plugins: Map<string, LoadedPlugin> = new Map();
  private initialized = false;

  /** Whether the runtime is running inside a Tauri window. */
  get isTauri(): boolean {
    return isTauri();
  }

  /** Initialize the runtime: fetch plugin list from host and load all enabled. */
  async init(): Promise<void> {
    if (this.initialized) return;

    if (!this.isTauri) {
      console.info("[plugin-runtime] not in Tauri — skipping plugin load");
      this.initialized = true;
      return;
    }

    const { invoke } = await import("@tauri-apps/api/core");

    let pluginList: PluginInfo[];
    try {
      pluginList = await invoke<PluginInfo[]>("list_enabled_plugins");
    } catch (err) {
      console.error("[plugin-runtime] failed to list plugins:", err);
      this.initialized = true;
      return;
    }

    for (const info of pluginList) {
      if (info.status !== "enabled") continue;
      await this.loadPlugin(info);
    }

    this.initialized = true;
    console.info(
      `[plugin-runtime] initialized — ${this.plugins.size} plugin(s) loaded`
    );
  }

  /** Load a single plugin: fetch its source, evaluate, instantiate. */
  private async loadPlugin(info: PluginInfo): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core");

    // Fetch entry source from host
    let source: string;
    try {
      source = await invoke<string>("get_plugin_source", { id: info.id });
    } catch (err) {
      console.error(`[plugin-runtime] failed to get source for ${info.id}:`, err);
      this.plugins.set(info.id, {
        info,
        instance: null,
        api: null,
        source: "",
        error: String(err),
      });
      return;
    }

    // Evaluate the plugin module in a controlled scope
    const api = this.createPluginAPI(info);
    try {
      const instance = await this.evaluatePlugin(source, info.id);
      if (instance) {
        this.plugins.set(info.id, {
          info,
          instance,
          api,
          source,
          error: null,
        });

        // Call onLoad if the plugin defines it
        if (instance.onLoad) {
          try {
            await instance.onLoad(api);
          } catch (err) {
            console.error(`[plugin-runtime] ${info.id}.onLoad error:`, err);
          }
        }
        console.info(`[plugin-runtime] loaded plugin: ${info.id} (${info.name})`);
      }
    } catch (err) {
      console.error(`[plugin-runtime] failed to evaluate ${info.id}:`, err);
      this.plugins.set(info.id, {
        info,
        instance: null,
        api,
        source,
        error: String(err),
      });
    }
  }

  /**
   * Evaluate a plugin's ES module source and extract its default export.
   *
   * Uses a Blob URL approach to load the plugin as an ES module. The module
   * must default-export an object implementing EasyMusicPlugin.
   */
  private async evaluatePlugin(
    source: string,
    pluginId: string
  ): Promise<EasyMusicPlugin | null> {
    // Wrap the source so we can capture the default export
    const moduleCode = `${source}\n//# sourceURL=easymusic-plugin://${pluginId}/entry.js`;

    const blob = new Blob([moduleCode], { type: "application/javascript" });
    const url = URL.createObjectURL(blob);

    try {
      const mod = await import(/* @vite-ignore */ url);
      if (mod.default && typeof mod.default === "object") {
        return mod.default as EasyMusicPlugin;
      }
      // If no default export but the module object looks like a plugin
      if (typeof mod.onLoad === "function" || typeof mod.onTrackChanged === "function") {
        return mod as unknown as EasyMusicPlugin;
      }
      console.warn(`[plugin-runtime] ${pluginId}: no valid plugin export found`);
      return null;
    } catch (err) {
      console.error(`[plugin-runtime] ${pluginId} eval error:`, err);
      return null;
    } finally {
      URL.revokeObjectURL(url);
    }
  }

  /** Create the permission-scoped API object for a plugin. */
  private createPluginAPI(info: PluginInfo): PluginAPI {
    const perms = new Set(info.permissions);
    const manifest: PluginManifest = {
      id: info.id,
      name: info.name,
      version: info.version,
      author: info.author,
      description: info.description,
      engine: "js",
      entry: "",
      permissions: info.permissions as Permission[],
      hooks: info.hooks as PluginHookName[],
    };

    const api: PluginAPI = {
      manifest,
      log: (...args: unknown[]) => {
        console.log(`[${info.id}]`, ...args);
      },
    };

    // Conditionally add methods based on permissions
    if (perms.has("library:read")) {
      // getCurrentTrack will be wired by the app context
      // The runtime stores a reference set by setAppContext
      if (this._getCurrentTrack) {
        api.getCurrentTrack = this._getCurrentTrack;
      }
    }

    if (perms.has("network:fetch")) {
      api.proxiedFetch = async (url, opts) => {
        // In a real implementation, this calls a Tauri command to proxy
        // For now, we use a simple fetch fallback
        try {
          const resp = await fetch(url, {
            method: opts?.method || "GET",
            headers: opts?.headers,
          });
          return await resp.text();
        } catch (err) {
          throw new Error(`proxiedFetch failed: ${err}`);
        }
      };
    }

    if (perms.has("playback:control") && this._playbackControl) {
      api.playbackControl = this._playbackControl;
    }

    return api;
  }

  // ── App context callbacks (set by the main app) ───────────────────

  private _getCurrentTrack: (() => Track | null) | null = null;
  private _playbackControl:
    | ((action: "play" | "pause" | "stop") => void)
    | null = null;

  /**
   * Set the app context callbacks. Called by the main app to provide
   * live state to plugins.
   */
  setAppContext(callbacks: {
    getCurrentTrack?: () => Track | null;
    playbackControl?: (action: "play" | "pause" | "stop") => void;
  }) {
    if (callbacks.getCurrentTrack) this._getCurrentTrack = callbacks.getCurrentTrack;
    if (callbacks.playbackControl) this._playbackControl = callbacks.playbackControl;
  }

  // ── Hook dispatchers ──────────────────────────────────────────────

  /** Dispatch onTrackChanged to all plugins subscribed to it. */
  dispatchTrackChanged(track: Track): void {
    this.dispatchToSubscribers("onTrackChanged", (plugin) => {
      if (plugin.instance?.onTrackChanged) {
        plugin.instance.onTrackChanged(track);
      }
    });
  }

  /** Dispatch onPlaybackStateChanged. */
  dispatchPlaybackStateChanged(payload: PlaybackStatePayload): void {
    this.dispatchToSubscribers("onPlaybackStateChanged", (plugin) => {
      if (plugin.instance?.onPlaybackStateChanged) {
        plugin.instance.onPlaybackStateChanged(payload);
      }
    });
  }

  /** Dispatch onLibraryScanned. */
  dispatchLibraryScanned(payload: LibraryScannedPayload): void {
    this.dispatchToSubscribers("onLibraryScanned", (plugin) => {
      if (plugin.instance?.onLibraryScanned) {
        plugin.instance.onLibraryScanned(payload);
      }
    });
  }

  /** Render custom UI panels for plugins that subscribe to customUIPanel. */
  renderUIPanels(container: HTMLElement): void {
    for (const [id, plugin] of this.plugins) {
      if (
        plugin.instance?.customUIPanel &&
        plugin.info.hooks.includes("customUIPanel")
      ) {
        try {
          const panel = document.createElement("div");
          panel.className = `plugin-panel plugin-panel-${id}`;
          container.appendChild(panel);
          plugin.instance.customUIPanel(panel);
        } catch (err) {
          console.error(`[plugin-runtime] ${id} customUIPanel error:`, err);
        }
      }
    }
  }

  /** Generic hook dispatch helper. */
  private dispatchToSubscribers(
    hookName: PluginHookName,
    fn: (plugin: LoadedPlugin) => void
  ): void {
    for (const [id, plugin] of this.plugins) {
      if (plugin.instance && plugin.info.hooks.includes(hookName)) {
        try {
          fn(plugin);
        } catch (err) {
          console.error(`[plugin-runtime] ${id} ${hookName} error:`, err);
        }
      }
    }
  }

  /** Reload all plugins from the host. */
  async reload(): Promise<void> {
    // Unload current plugins
    for (const [id, plugin] of this.plugins) {
      if (plugin.instance?.onUnload) {
        try {
          plugin.instance.onUnload();
        } catch (err) {
          console.error(`[plugin-runtime] ${id} onUnload error:`, err);
        }
      }
    }
    this.plugins.clear();
    this.initialized = false;
    await this.init();
  }

  /** Get info about all loaded plugins. */
  getLoadedPlugins(): { id: string; name: string; error: string | null }[] {
    return Array.from(this.plugins.values()).map((p) => ({
      id: p.info.id,
      name: p.info.name,
      error: p.error,
    }));
  }

  /** Check if any plugin subscribes to a hook. */
  hasSubscribers(hookName: PluginHookName): boolean {
    for (const plugin of this.plugins.values()) {
      if (plugin.instance && plugin.info.hooks.includes(hookName)) return true;
    }
    return false;
  }
}

/** Global singleton instance. */
let _runtime: PluginRuntime | null = null;

/** Get the singleton plugin runtime. */
export function getPluginRuntime(): PluginRuntime {
  if (!_runtime) {
    _runtime = new PluginRuntime();
  }
  return _runtime;
}
