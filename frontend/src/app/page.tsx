"use client";

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

/**
 * EasyMusic 启动占位页。
 * 在 Tauri 窗口内运行时，会通过 invoke 调用 Rust 命令 `greet`，
 * 验证 frontend → Tauri → easy-music-core 全链路已打通。
 * 在普通浏览器中打开时优雅降级（只显示静态文案）。
 */
export default function Home() {
  const [greeting, setGreeting] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      if (!isTauri()) {
        setGreeting("在浏览器中预览 — 请在 Tauri 窗口内运行以启用 Rust 桥接");
        return;
      }
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const msg: string = await invoke("greet", { name: "EasyMusic" });
        if (!cancelled) setGreeting(msg);
      } catch (err) {
        if (!cancelled) setGreeting(`Rust 桥接调用失败: ${String(err)}`);
      }
    }
    load();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="flex flex-1 flex-col items-center justify-center gap-4 bg-zinc-950 text-zinc-50">
      <h1 className="text-4xl font-semibold tracking-tight">🎵 EasyMusic</h1>
      <p className="text-zinc-400">Tauri + Next.js + Rust 脚手架已就绪</p>
      {greeting && <p className="text-sm text-indigo-400">{greeting}</p>}
    </main>
  );
}
