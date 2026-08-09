import { AppShell } from "@/components/AppShell";

/**
 * EasyMusic — root page.
 * Renders the full single-page application shell (sidebar + content + now
 * playing bar). All views are rendered client-side via Zustand navigation
 * state, which keeps the app inside the Tauri webview feeling instant.
 */
export default function Home() {
  return <AppShell />;
}
