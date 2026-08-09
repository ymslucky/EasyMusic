"use client";

import { useMemo } from "react";
import { Play } from "lucide-react";
import { AlbumArt } from "@/components/ui/AlbumArt";
import { useStore } from "@/lib/store";
import { cn, formatTime } from "@/lib/utils";
import type { Artist } from "@/lib/types";

function artistHue(name: string): number {
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) % 360;
  return h;
}

export function ArtistsView() {
  const artists = useStore((s) => s.artists);
  const navigate = useStore((s) => s.navigate);

  return (
    <div className="flex h-full flex-col">
      <div className="px-6 py-4 border-b border-border">
        <h2 className="text-xl font-bold">Artists</h2>
        <p className="text-sm text-text-muted mt-0.5">{artists.length} artists</p>
      </div>
      <div className="flex-1 overflow-y-auto p-6">
        <div className="grid grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-5">
          {artists.map((artist: Artist) => (
            <button
              key={artist.id}
              onClick={() => navigate({ name: "artist", key: artist.name })}
              className="group flex flex-col items-center gap-2 text-center rounded-lg p-3 hover:bg-bg-hover transition-colors"
            >
              <AlbumArt
                title={artist.name}
                hue={artistHue(artist.name)}
                size={140}
                rounded="rounded-full"
                className="w-full aspect-square h-auto"
              />
              <div className="truncate text-sm font-medium w-full">{artist.name}</div>
              <div className="text-xs text-text-muted">{artist.track_count} tracks</div>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

export function ArtistDetailView({ artistKey }: { artistKey: string }) {
  const artists = useStore((s) => s.artists);
  const albums = useStore((s) => s.albums);
  const tracks = useStore((s) => s.tracks);
  const playTrack = useStore((s) => s.playTrack);
  const currentTrackId = useStore((s) => s.currentTrackId);
  const navigate = useStore((s) => s.navigate);

  const artist = artists.find((a) => a.name === artistKey);
  const artistAlbums = useMemo(
    () => albums.filter((al) => al.artist === artistKey),
    [albums, artistKey],
  );

  if (!artist) {
    return (
      <div className="flex h-full items-center justify-center text-text-muted">
        Artist not found.{" "}
        <button className="ml-2 text-accent" onClick={() => navigate({ name: "artists" })}>
          Back
        </button>
      </div>
    );
  }

  const allTracks = tracks.filter((t) => t.artist === artistKey);
  const playAll = () => {
    if (allTracks.length > 0) playTrack(allTracks[0]!, allTracks);
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-end gap-6 px-6 py-6 border-b border-border">
        <AlbumArt title={artist.name} hue={artistHue(artist.name)} size={180} rounded="rounded-full" />
        <div className="flex flex-col gap-2 pb-2 min-w-0">
          <span className="text-xs uppercase tracking-wider text-text-muted">Artist</span>
          <h1 className="text-4xl font-bold truncate">{artist.name}</h1>
          <p className="text-text-secondary">
            {artist.track_count} tracks · {artist.album_count} albums
          </p>
          <div className="mt-3">
            <button
              onClick={playAll}
              className="inline-flex items-center gap-2 bg-accent text-zinc-950 font-semibold rounded-full h-10 px-6 hover:bg-accent-hover transition-colors"
            >
              <Play size={16} fill="currentColor" /> Play
            </button>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6 space-y-8">
        {artistAlbums.map((album) => {
          const albumTracks = tracks.filter((t) => t.album === album.title && t.artist === artistKey);
          if (albumTracks.length === 0) return null;
          return (
            <div key={album.id}>
              <button
                onClick={() => navigate({ name: "album", key: album.title })}
                className="flex items-center gap-3 mb-2 text-left hover:opacity-80"
              >
                <AlbumArt title={album.title} hue={artistHue(album.title)} size={40} />
                <div>
                  <div className="text-sm font-semibold">{album.title}</div>
                  <div className="text-xs text-text-muted">{albumTracks.length} tracks</div>
                </div>
              </button>
              {albumTracks.map((t, i) => (
                <div
                  key={t.id}
                  className="flex items-center gap-4 px-2 h-11 group hover:bg-bg-hover rounded cursor-default"
                  onDoubleClick={() => playTrack(t, albumTracks)}
                >
                  <span className="w-6 text-right text-sm text-text-muted">{i + 1}</span>
                  <div className="flex-1 min-w-0">
                    <span className={cn("truncate text-sm", t.id === currentTrackId ? "text-accent" : "text-text")}>
                      {t.title}
                    </span>
                  </div>
                  <span className="text-xs text-text-muted tabular-nums">{formatTime(t.duration_secs)}</span>
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
}
