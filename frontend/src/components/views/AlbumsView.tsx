"use client";

import { Play } from "lucide-react";
import { AlbumArt } from "@/components/ui/AlbumArt";
import { useStore } from "@/lib/store";
import { cn, formatTime } from "@/lib/utils";
import type { Album } from "@/lib/types";

function albumHue(title: string): number {
  let h = 0;
  for (let i = 0; i < title.length; i++) h = (h * 31 + title.charCodeAt(i)) % 360;
  return h;
}

export function AlbumsView() {
  const albums = useStore((s) => s.albums);
  const navigate = useStore((s) => s.navigate);

  return (
    <div className="flex h-full flex-col">
      <div className="px-6 py-4 border-b border-border">
        <h2 className="text-xl font-bold">Albums</h2>
        <p className="text-sm text-text-muted mt-0.5">{albums.length} albums</p>
      </div>
      <div className="flex-1 overflow-y-auto p-6">
        <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-5">
          {albums.map((album) => (
            <button
              key={album.id}
              onClick={() => navigate({ name: "album", key: album.title })}
              className="group flex flex-col gap-2 text-left rounded-lg p-2 hover:bg-bg-hover transition-colors"
            >
              <div className="relative">
                <AlbumArt
                  title={album.title}
                  hue={albumHue(album.title)}
                  size={160}
                  rounded="rounded-lg"
                  className="w-full aspect-square h-auto"
                />
                <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                  <div className="bg-accent text-zinc-950 rounded-full w-11 h-11 flex items-center justify-center shadow-lg">
                    <Play size={18} fill="currentColor" />
                  </div>
                </div>
              </div>
              <div className="truncate text-sm font-medium">{album.title}</div>
              <div className="truncate text-xs text-text-muted">{album.artist ?? "Unknown"}</div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

export function AlbumDetailView({ albumKey }: { albumKey: string }) {
  const albums = useStore((s) => s.albums);
  const tracks = useStore((s) => s.tracks);
  const playTrack = useStore((s) => s.playTrack);
  const currentTrackId = useStore((s) => s.currentTrackId);
  const navigate = useStore((s) => s.navigate);

  const album: Album | undefined = albums.find((a) => a.title === albumKey);
  if (!album) {
    return (
      <div className="flex h-full items-center justify-center text-text-muted">
        Album not found.{" "}
        <button className="ml-2 text-accent" onClick={() => navigate({ name: "albums" })}>
          Back
        </button>
      </div>
    );
  }

  const albumTracks = tracks.filter((t) => t.album === album.title);
  const totalSecs = albumTracks.reduce((sum, t) => sum + t.duration_secs, 0);
  const mins = Math.round(totalSecs / 60);

  const playAlbum = () => {
    if (albumTracks.length > 0) playTrack(albumTracks[0]!, albumTracks);
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-end gap-6 px-6 py-6 border-b border-border">
        <AlbumArt title={album.title} hue={albumHue(album.title)} size={180} rounded="rounded-xl" />
        <div className="flex flex-col gap-2 pb-2 min-w-0">
          <span className="text-xs uppercase tracking-wider text-text-muted">Album</span>
          <h1 className="text-4xl font-bold truncate">{album.title}</h1>
          <p className="text-text-secondary">
            {album.artist ?? "Unknown Artist"} · {albumTracks.length} tracks · {mins} min
          </p>
          <div className="mt-3">
            <button
              onClick={playAlbum}
              className="inline-flex items-center gap-2 bg-accent text-zinc-950 font-semibold rounded-full h-10 px-6 hover:bg-accent-hover transition-colors"
            >
              <Play size={16} fill="currentColor" /> Play
            </button>
          </div>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto">
        {albumTracks.map((t, i) => (
          <div
            key={t.id}
            className="flex items-center gap-4 px-6 h-12 group hover:bg-bg-hover cursor-default"
            onDoubleClick={() => playTrack(t, albumTracks)}
          >
            <span className="w-6 text-right text-sm text-text-muted">{i + 1}</span>
            <div className="flex-1 min-w-0">
              <div className={cn("truncate text-sm", t.id === currentTrackId ? "text-accent" : "text-text")}>
                {t.title}
              </div>
              <div className="truncate text-xs text-text-muted">{t.artist}</div>
            </div>
            <span className="text-sm text-text-muted tabular-nums">{formatTime(t.duration_secs)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
