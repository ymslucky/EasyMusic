"use client";

import { useVirtualizer } from "@tanstack/react-virtual";
import { useMemo, useRef } from "react";
import {
  ChevronDown,
  ChevronUp,
  Clock,
  Play,
  Search,
} from "lucide-react";
import { useStore, type SortKey } from "@/lib/store";
import { cn, formatTime } from "@/lib/utils";
import type { Track } from "@/lib/types";

const COLUMNS: { key: SortKey | "index"; label: string; className: string }[] = [
  { key: "index", label: "#", className: "w-12 text-right" },
  { key: "title", label: "Title", className: "flex-1" },
  { key: "artist", label: "Artist", className: "w-44" },
  { key: "album", label: "Album", className: "w-52" },
  { key: "duration_secs", label: "", className: "w-16 text-right" },
];

const ROW_HEIGHT = 44;
const HEADER_HEIGHT = 40;

export function LibraryView() {
  const tracks = useStore((s) => s.tracks);
  const filter = useStore((s) => s.filter);
  const setFilter = useStore((s) => s.setFilter);
  const getFilteredTracks = useStore((s) => s.getFilteredTracks);
  const playTrack = useStore((s) => s.playTrack);
  const currentTrackId = useStore((s) => s.currentTrackId);
  const playbackState = useStore((s) => s.playbackState);

  const filtered = useMemo(
    () => getFilteredTracks(),
    [getFilteredTracks, tracks, filter],
  );

  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: filtered.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 16,
  });

  const totalHeight = virtualizer.getTotalSize();

  const toggleSort = (key: SortKey) => {
    if (filter.sortKey === key) {
      setFilter({ sortDir: filter.sortDir === "asc" ? "desc" : "asc" });
    } else {
      setFilter({ sortKey: key, sortDir: "asc" });
    }
  };

  const handlePlay = (track: Track) => {
    playTrack(track, filtered);
  };

  return (
    <div className="flex h-full flex-col">
      {/* Toolbar */}
      <div className="flex items-center gap-3 px-6 py-3 border-b border-border">
        <div className="relative flex-1 max-w-sm">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
          <input
            type="search"
            placeholder="Search tracks, artists, albums…"
            value={filter.search}
            onChange={(e) => setFilter({ search: e.target.value })}
            className="w-full h-9 pl-9 pr-3 rounded-md bg-bg-elevated border border-border text-sm text-text placeholder:text-text-muted outline-none focus:border-accent/50"
          />
        </div>
        <span className="text-xs text-text-muted">
          {filtered.length} of {tracks.length} tracks
        </span>
      </div>

      {/* Header */}
      <div
        className="flex items-center gap-4 px-6 text-xs uppercase tracking-wider text-text-muted border-b border-border"
        style={{ height: HEADER_HEIGHT }}
      >
        {COLUMNS.map((col) => {
          const sortable = col.key !== "index";
          const isActive = filter.sortKey === col.key;
          return (
            <button
              key={col.key}
              disabled={!sortable}
              onClick={() => sortable && toggleSort(col.key as SortKey)}
              className={cn(
                "flex items-center gap-1",
                col.className,
                sortable && "hover:text-text",
                isActive && "text-accent",
              )}
            >
              {col.label || (col.key === "duration_secs" && <Clock size={13} />)}
              {isActive && (filter.sortDir === "asc" ? <ChevronUp size={12} /> : <ChevronDown size={12} />)}
            </button>
          );
        })}
      </div>

      {/* Virtualized rows */}
      <div ref={parentRef} className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="flex h-full items-center justify-center text-text-muted text-sm">
            {tracks.length === 0
              ? "No tracks in library. Add a directory in Settings."
              : "No tracks match your search."}
          </div>
        ) : (
          <div className="relative px-6" style={{ height: totalHeight }}>
            {virtualizer.getVirtualItems().map((vRow) => {
              const track = filtered[vRow.index]!;
              const isCurrent = track.id === currentTrackId;
              const isPlaying = isCurrent && playbackState === "Playing";
              return (
                <div
                  key={track.id}
                  className="absolute left-6 right-6 flex items-center gap-4 group cursor-default"
                  style={{ height: ROW_HEIGHT, top: vRow.start }}
                  onDoubleClick={() => handlePlay(track)}
                >
                  {/* index / play button */}
                  <div className="w-12 text-right text-sm tabular-nums text-text-muted group-hover:hidden">
                    {isPlaying ? (
                      <EqualizerBars />
                    ) : (
                      <span className={cn(isCurrent && "text-accent")}>{vRow.index + 1}</span>
                    )}
                  </div>
                  <button
                    className="w-12 text-right hidden group-hover:flex justify-end items-center"
                    onClick={() => handlePlay(track)}
                    aria-label={`Play ${track.title}`}
                  >
                    <Play size={14} className="text-text" fill="currentColor" />
                  </button>
                  <div className="flex-1 min-w-0">
                    <div className={cn("truncate text-sm", isCurrent ? "text-accent" : "text-text")}>
                      {track.title}
                    </div>
                  </div>
                  <div className="w-44 truncate text-sm text-text-secondary">{track.artist}</div>
                  <div className="w-52 truncate text-sm text-text-secondary">{track.album ?? "—"}</div>
                  <div className="w-16 text-right text-sm tabular-nums text-text-muted">
                    {formatTime(track.duration_secs)}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

function EqualizerBars() {
  return (
    <span className="inline-flex items-end gap-0.5 h-3.5" aria-label="Now playing">
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className="w-0.5 bg-accent"
          style={{
            height: "100%",
            animation: `eq 0.9s ease-in-out ${i * 0.18}s infinite alternate`,
          }}
        />
      ))}
      <style>{`@keyframes eq { from { transform: scaleY(0.25) } to { transform: scaleY(1) } }`}</style>
    </span>
  );
}
