import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Tauri serves static assets — no server required.
  output: "export",
  images: {
    unoptimized: true,
  },
  // The monorepo root and frontend/ each have a package-lock.json; pin the
  // Turbopack project root to this app directory to silence the
  // "inferred your workspace root" warning.
  turbopack: {
    root: __dirname,
  },
};

export default nextConfig;
