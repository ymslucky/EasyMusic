"use client";

import { cn } from "@/lib/utils";
import type { ButtonHTMLAttributes, ReactNode } from "react";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  /** Tooltip on hover. */
  label: string;
  /** When true, render with accent color. */
  active?: boolean;
}

export function IconButton({
  label,
  active,
  className,
  children,
  ...props
}: IconButtonProps) {
  return (
    <button
      aria-label={label}
      title={label}
      className={cn(
        "inline-flex items-center justify-center rounded-full transition-colors duration-150 outline-none focus-visible:ring-2 focus-visible:ring-accent/50 disabled:opacity-40 disabled:cursor-not-allowed",
        active
          ? "text-accent"
          : "text-text-secondary hover:text-text hover:bg-bg-hover",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
