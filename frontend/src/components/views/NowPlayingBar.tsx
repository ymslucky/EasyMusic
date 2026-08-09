"use client";

import {
  Pause,
  Play,
  Repeat,
  Repeat1,
  Shuffle,
  SkipBack,
  SkipForward,
  Volume1,
  Volume2,
  VolumeX,
} from "lucide-react";
import { AlbumArt } from "@/components/ui/AlbumArt";
import { IconButton } from "@/components/ui/IconButton";
import { Slider } from "@/components/ui/Slider";
import { useStore } from "@/lib/store";
import { formatTime } from "@/lib/utils";

export function NowPlayingBar() {
  const currentTrackId = useStore((s) => s.currentTrackId);
  const tracksById = useStore((s) => s.tracksById);
  const playbackState = useStore((s) => s.playbackState);
  const positionSecs = useStore((s) => s.positionSecs);
  const volume = useStore((s) => s.volume);
  const repeatMode = useStore((s) => s.repeatMode);
  const shuffle = useStore((s) => s.shuffle);

  const togglePlay = useStore((s) => s.togglePlay);
  const next = useStore((s) => s.next);
  const prev = useStore((s) => s.prev);
  const seek = useStore((s) => s.seek);
  const setVolume = useStore((s) => s.setVolume);
  const toggleRepeat = useStore((s) => s.toggleRepeat);
  const toggleShuffle = useStore((s) => s.toggleShuffle);

  const track = currentTrackId ? tracksById[currentTrackId] : null;
  const isPlaying = playbackState === "Playing";
  const duration = track?.duration_secs ?? 0;

  const VolIcon = volume === 0 ? VolumeX : volume < 0.5 ? Volume1 : Volume2;

  return (
    <footer
      className="flex items-center gap-4 px-4 border-t border-border bg-bg-elevated"
      style={{ height: "var(--nowplaying-h)" }}
    >
      {/* Left: track info */}
      <div className="flex items-center gap-3 w-[280px] min-w-0">
        {track ? (
          <>
            <AlbumArt
              title={track.album ?? track.title}
              hue={(track.title.charCodeAt(0) ?? 0) * 13}
              size={56}
              rounded="rounded-md"
            />
            <div className="min-w-0">
              <div className="truncate text-sm font-medium">{track.title}</div>
              <div className="truncate text-xs text-text-muted">{track.artist}</div>
            </div>
          </>
        ) : (
          <div className="flex items-center gap-3 text-text-muted">
            <div className="w-14 h-14 rounded-md bg-bg-active flex items-center justify-center text-2xl">
              ♪
            </div>
            <div className="text-sm">Not playing</div>
          </div>
        )}
      </div>

      {/* Center: transport + seek */}
      <div className="flex-1 flex flex-col items-center gap-1.5 max-w-2xl mx-auto">
        <div className="flex items-center gap-2">
          <IconButton
            label="Shuffle"
            onClick={toggleShuffle}
            active={shuffle}
            className="w-8 h-8"
          >
            <Shuffle size={15} />
          </IconButton>
          <IconButton
            label="Previous"
            onClick={prev}
            disabled={!track}
            className="w-9 h-9"
          >
            <SkipBack size={18} fill="currentColor" />
          </IconButton>
          <button
            onClick={togglePlay}
            disabled={!track}
            aria-label={isPlaying ? "Pause" : "Play"}
            className="w-10 h-10 rounded-full bg-text text-zinc-950 flex items-center justify-center hover:scale-105 transition-transform disabled:opacity-40 disabled:hover:scale-100"
          >
            {isPlaying ? (
              <Pause size={18} fill="currentColor" />
            ) : (
              <Play size={18} fill="currentColor" className="translate-x-0.5" />
            )}
          </button>
          <IconButton label="Next" onClick={next} disabled={!track} className="w-9 h-9">
            <SkipForward size={18} fill="currentColor" />
          </IconButton>
          <IconButton
            label={`Repeat: ${repeatMode}`}
            onClick={toggleRepeat}
            active={repeatMode !== "off"}
            className="w-8 h-8"
          >
            {repeatMode === "one" ? <Repeat1 size={15} /> : <Repeat size={15} />}
          </IconButton>
        </div>
        <div className="flex items-center gap-2 w-full">
          <span className="text-[10px] tabular-nums text-text-muted w-10 text-right">
            {formatTime(positionSecs)}
          </span>
          <Slider
            value={Math.min(positionSecs, duration || 0)}
            max={duration || 1}
            step={1}
            onChange={(v) => seek(v)}
            ariaLabel="Seek"
            className="flex-1"
          />
          <span className="text-[10px] tabular-nums text-text-muted w-10">
            {formatTime(duration)}
          </span>
        </div>
      </div>

      {/* Right: volume */}
      <div className="flex items-center gap-2 w-[200px] justify-end">
        <IconButton
          label={`Volume ${Math.round(volume * 100)}%`}
          onClick={() => setVolume(volume === 0 ? 0.8 : 0)}
          className="w-8 h-8"
        >
          <VolIcon size={16} />
        </IconButton>
        <Slider
          value={volume}
          onChange={(v) => setVolume(v)}
          ariaLabel="Volume"
          className="w-24"
        />
      </div>
    </footer>
  );
}
