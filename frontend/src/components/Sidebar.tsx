"use client";

import {
  Disc3,
  Library,
  ListMusic,
  Settings as SettingsIcon,
  Users,
} from "lucide-react";
import { useStore, type AppState } from "@/lib/store";
import { cn } from "@/lib/utils";

interface NavItem {
  label: string;
  icon: React.ReactNode;
  view: AppState["view"];
  match: (v: AppState["view"]) => boolean;
}

const NAV: NavItem[] = [
  {
    label: "Library",
    icon: <Library size={18} />,
    view: { name: "library" },
    match: (v) => v.name === "library",
  },
  {
    label: "Albums",
    icon: <Disc3 size={18} />,
    view: { name: "albums" },
    match: (v) => v.name === "albums" || v.name === "album",
  },
  {
    label: "Artists",
    icon: <Users size={18} />,
    view: { name: "artists" },
    match: (v) => v.name === "artists" || v.name === "artist",
  },
  {
    label: "Playlists",
    icon: <ListMusic size={18} />,
    view: { name: "playlists" },
    match: (v) =>
      v.name === "playlists" || v.name === "playlist",
  },
];

export function Sidebar() {
  const view = useStore((s) => s.view);
  const navigate = useStore((s) => s.navigate);
  const playlists = useStore((s) => s.playlists);
  const tracks = useStore((s) => s.tracks);

  return (
    <aside
      className="flex flex-col bg-bg-elevated border-r border-border shrink-0"
      style={{ width: "var(--sidebar-w)" }}
    >
      {/* Brand */}
      <div
        className="flex items-center gap-2.5 px-5 border-b border-border"
        style={{ height: "var(--header-h)" }}
      >
        <div className="w-7 h-7 rounded-md bg-gradient-to-br from-accent to-emerald-700 flex items-center justify-center text-zinc-950 font-bold">
          ♪
        </div>
        <span className="font-bold text-sm tracking-tight">EasyMusic</span>
      </div>

      {/* Nav */}
      <nav className="flex-1 overflow-y-auto px-3 py-3">
        <div className="space-y-0.5">
          {NAV.map((item) => (
            <button
              key={item.label}
              onClick={() => navigate(item.view)}
              className={cn(
                "flex items-center gap-3 w-full h-9 px-3 rounded-md text-sm transition-colors",
                item.match(view)
                  ? "bg-bg-active text-text font-medium"
                  : "text-text-secondary hover:text-text hover:bg-bg-hover",
              )}
            >
              <span className={cn(item.match(view) && "text-accent")}>
                {item.icon}
              </span>
              {item.label}
            </button>
          ))}
        </div>

        {/* Playlists quick list */}
        {playlists.length > 0 && (
          <div className="mt-6">
            <div className="px-3 mb-1 text-[10px] uppercase tracking-wider text-text-muted">
              Your Playlists
            </div>
            <div className="space-y-0.5">
              {playlists.map((pl) => (
                <button
                  key={pl.id}
                  onClick={() => navigate({ name: "playlist", id: pl.id })}
                  className={cn(
                    "flex items-center gap-2 w-full h-8 px-3 rounded-md text-xs truncate transition-colors",
                    view.name === "playlist" && view.id === pl.id
                      ? "bg-bg-active text-text"
                      : "text-text-muted hover:text-text hover:bg-bg-hover",
                  )}
                >
                  <ListMusic size={13} className="shrink-0" />
                  <span className="truncate">{pl.name}</span>
                </button>
              ))}
            </div>
          </div>
        )}
      </nav>

      {/* Footer: library stats + settings */}
      <div className="border-t border-border px-3 py-2 space-y-0.5">
        <div className="px-3 py-1.5 text-[10px] text-text-muted">
          {tracks.length} tracks in library
        </div>
        <button
          onClick={() => navigate({ name: "settings" })}
          className={cn(
            "flex items-center gap-3 w-full h-9 px-3 rounded-md text-sm transition-colors",
            view.name === "settings"
              ? "bg-bg-active text-text font-medium"
              : "text-text-secondary hover:text-text hover:bg-bg-hover",
          )}
        >
          <SettingsIcon size={18} />
          Settings
        </button>
      </div>
    </aside>
  );
}
