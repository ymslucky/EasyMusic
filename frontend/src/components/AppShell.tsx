"use client";

import { useEffect } from "react";
import { ChevronLeft } from "lucide-react";
import { Sidebar } from "@/components/Sidebar";
import { NowPlayingBar } from "@/components/views/NowPlayingBar";
import { LibraryView } from "@/components/views/LibraryView";
import { AlbumsView, AlbumDetailView } from "@/components/views/AlbumsView";
import { ArtistsView, ArtistDetailView } from "@/components/views/ArtistsView";
import {
  PlaylistsView,
  PlaylistDetailView,
} from "@/components/views/PlaylistsView";
import { SettingsView } from "@/components/views/SettingsView";
import { useStore } from "@/lib/store";

export function AppShell() {
  const view = useStore((s) => s.view);
  const navigate = useStore((s) => s.navigate);
  const init = useStore((s) => s.init);
  const loading = useStore((s) => s.loading);
  const error = useStore((s) => s.error);

  useEffect(() => {
    void init();
  }, [init]);

  // Determine whether to show a back button (detail views).
  const showBack =
    view.name === "album" ||
    view.name === "artist" ||
    view.name === "playlist";

  const backTarget = () => {
    if (view.name === "album") return { name: "albums" } as const;
    if (view.name === "artist") return { name: "artists" } as const;
    if (view.name === "playlist") return { name: "playlists" } as const;
    return { name: "library" } as const;
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-bg text-text">
      <Sidebar />

      <div className="flex flex-1 flex-col min-w-0">
        {/* Optional top bar with back button for detail views */}
        {showBack && (
          <div
            className="flex items-center border-b border-border bg-bg-elevated px-3"
            style={{ height: "var(--header-h)" }}
          >
            <button
              onClick={() => navigate(backTarget())}
              className="flex items-center gap-1 h-8 px-3 rounded-md text-sm text-text-secondary hover:text-text hover:bg-bg-hover"
            >
              <ChevronLeft size={16} /> Back
            </button>
          </div>
        )}

        {/* Main content area */}
        <main className="flex-1 min-h-0 overflow-hidden bg-bg">
          {loading ? (
            <div className="flex h-full items-center justify-center text-text-muted">
              Loading library…
            </div>
          ) : error ? (
            <div className="flex h-full items-center justify-center text-danger">
              {error}
            </div>
          ) : (
            <RenderView />
          )}
        </main>

        {/* Persistent bottom playback bar */}
        <NowPlayingBar />
      </div>
    </div>
  );

  function RenderView() {
    switch (view.name) {
      case "library":
        return <LibraryView />;
      case "albums":
        return <AlbumsView />;
      case "album":
        return <AlbumDetailView albumKey={view.key} />;
      case "artists":
        return <ArtistsView />;
      case "artist":
        return <ArtistDetailView artistKey={view.key} />;
      case "playlists":
        return <PlaylistsView />;
      case "playlist":
        return <PlaylistDetailView playlistId={view.id} />;
      case "settings":
        return <SettingsView />;
      default:
        return <LibraryView />;
    }
  }
}
