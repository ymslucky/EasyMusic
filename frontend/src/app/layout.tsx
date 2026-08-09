import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "EasyMusic",
  description: "跨平台桌面音乐播放器 — Tauri + Next.js + Rust",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="zh-CN" className="h-full antialiased">
      <body className="min-h-full flex flex-col">{children}</body>
    </html>
  );
}
