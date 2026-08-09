"use client";

import { cn } from "@/lib/utils";

interface AlbumArtProps {
  title: string;
  /** Deterministic hue 0-360 for the gradient. */
  hue: number;
  size?: number;
  rounded?: string;
  className?: string;
}

/**
 * CSS-only album art placeholder.
 * Real cover-art loading will swap this for an <img>; for now every album
 * gets a deterministic two-stop gradient seeded from its title so the grid
 * is visually distinct without binary assets.
 */
export function AlbumArt({
  title,
  hue,
  size = 48,
  rounded = "rounded-md",
  className,
}: AlbumArtProps) {
  const h2 = (hue + 40) % 360;
  const initials = title
    .split(/\s+/)
    .slice(0, 2)
    .map((w) => w[0]?.toUpperCase() ?? "")
    .join("");
  return (
    <div
      className={cn(
        "flex items-center justify-center shrink-0 overflow-hidden select-none",
        rounded,
        className,
      )}
      style={{
        width: size,
        height: size,
        background: `linear-gradient(135deg, hsl(${hue} 55% 45%), hsl(${h2} 60% 35%))`,
      }}
      aria-hidden
    >
      <span
        className="font-bold text-white/80 tracking-tight"
        style={{ fontSize: size * 0.32 }}
      >
        {initials || "♪"}
      </span>
    </div>
  );
}
