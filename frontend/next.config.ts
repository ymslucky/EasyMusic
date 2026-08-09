import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Tauri serves static assets — no server required.
  output: "export",
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
