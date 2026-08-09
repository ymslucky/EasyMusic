"use client";

import { useCallback, useRef } from "react";
import { cn } from "@/lib/utils";

interface SliderProps {
  value: number;
  min?: number;
  max?: number;
  step?: number;
  onChange: (value: number) => void;
  onCommit?: (value: number) => void;
  className?: string;
  ariaLabel: string;
  /** Render the filled portion in accent color (default true). */
  accentFill?: boolean;
}

/**
 * Accessible range slider with a custom-drawn track and fill.
 * Uses a native <input type="range"> under the hood for keyboard support,
 * overlaid with a div that renders the progress fill.
 */
export function Slider({
  value,
  min = 0,
  max = 1,
  step = 0.01,
  onChange,
  onCommit,
  className,
  ariaLabel,
  accentFill = true,
}: SliderProps) {
  const pct = max > min ? ((value - min) / (max - min)) * 100 : 0;

  const handleRef = useRef<HTMLInputElement>(null);
  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange(Number(e.target.value));
    },
    [onChange],
  );
  const handleKeyUp = useCallback(() => {
    onCommit?.(value);
  }, [onCommit, value]);

  return (
    <div className={cn("group relative flex items-center h-4 w-full", className)}>
      {/* track */}
      <div className="absolute inset-x-0 top-1/2 h-1 -translate-y-1/2 rounded-full bg-border overflow-hidden">
        <div
          className={cn(
            "h-full rounded-full",
            accentFill ? "bg-accent group-hover:bg-accent-hover" : "bg-text",
          )}
          style={{ width: `${pct}%` }}
        />
      </div>
      {/* native input (transparent, drives interaction) */}
      <input
        ref={handleRef}
        type="range"
        aria-label={ariaLabel}
        aria-valuetext={`${pct.toFixed(0)}%`}
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={handleChange}
        onKeyUp={handleKeyUp}
        onTouchEnd={() => onCommit?.(value)}
        onMouseUp={() => onCommit?.(value)}
        className="absolute inset-0 z-10 w-full opacity-0 cursor-pointer"
      />
    </div>
  );
}
