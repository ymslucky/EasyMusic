"use client";

import { useState } from "react";
import {
  FolderPlus,
  Music,
  Palette,
  Plug,
  RefreshCw,
  Trash2,
  FolderInput,
  ChevronDown,
  ChevronUp,
  Shield,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { useStore } from "@/lib/store";
import type { PluginInfo } from "@/lib/types";

export function SettingsView() {
  const libraryDirs = useStore((s) => s.libraryDirs);
  const plugins = useStore((s) => s.plugins);
  const addLibraryDir = useStore((s) => s.addLibraryDir);
  const removeLibraryDir = useStore((s) => s.removeLibraryDir);
  const togglePlugin = useStore((s) => s.togglePlugin);
  const reloadPlugins = useStore((s) => s.reloadPlugins);
  const installPlugin = useStore((s) => s.installPlugin);
  const uninstallPlugin = useStore((s) => s.uninstallPlugin);

  const [newPath, setNewPath] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [installPath, setInstallPath] = useState("");
  const [expandedPlugin, setExpandedPlugin] = useState<string | null>(null);

  const handleAddDir = async () => {
    const path = newPath.trim();
    const label = newLabel.trim() || path.split("/").pop() || "Directory";
    if (!path) return;
    await addLibraryDir(path, label);
    setNewPath("");
    setNewLabel("");
  };

  const handleInstallPlugin = async () => {
    const path = installPath.trim();
    if (!path) return;
    try {
      await installPlugin(path);
      setInstallPath("");
    } catch (err) {
      console.error("Failed to install plugin:", err);
    }
  };

  const handleReload = async () => {
    await reloadPlugins();
  };

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-3xl mx-auto px-6 py-8 space-y-10">
        <header>
          <h2 className="text-2xl font-bold">设置</h2>
          <p className="text-sm text-text-muted mt-1">
            管理音乐库目录、外观和插件。
          </p>
        </header>

        {/* Library directories */}
        <Section
          icon={<Music size={18} />}
          title="音乐库目录"
          desc="EasyMusic 扫描以下目录中的音频文件。"
        >
          <div className="space-y-2">
            {libraryDirs.map((dir) => (
              <div
                key={dir.id}
                className="flex items-center gap-3 rounded-md border border-border bg-bg-elevated px-4 py-3"
              >
                <FolderPlus size={16} className="text-text-muted" />
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium truncate">{dir.label}</div>
                  <div className="text-xs text-text-muted truncate">{dir.path}</div>
                </div>
                <span className="text-xs px-2 py-0.5 rounded-full bg-accent/10 text-accent">
                  {dir.enabled ? "活跃" : "已禁用"}
                </span>
                <button
                  onClick={() => removeLibraryDir(dir.id)}
                  className="text-text-muted hover:text-danger"
                  aria-label={`移除 ${dir.label}`}
                >
                  <Trash2 size={15} />
                </button>
              </div>
            ))}
            {libraryDirs.length === 0 && (
              <p className="text-sm text-text-muted py-2">尚未添加目录。</p>
            )}
          </div>

          <div className="mt-4 flex gap-2 items-end">
            <div className="flex-1">
              <label className="block text-xs text-text-muted mb-1">路径</label>
              <input
                type="text"
                placeholder="/home/user/Music"
                value={newPath}
                onChange={(e) => setNewPath(e.target.value)}
                className="w-full h-9 px-3 rounded-md bg-bg-elevated border border-border text-sm outline-none focus:border-accent/50"
              />
            </div>
            <div className="w-40">
              <label className="block text-xs text-text-muted mb-1">标签</label>
              <input
                type="text"
                placeholder="Music"
                value={newLabel}
                onChange={(e) => setNewLabel(e.target.value)}
                className="w-full h-9 px-3 rounded-md bg-bg-elevated border border-border text-sm outline-none focus:border-accent/50"
              />
            </div>
            <Button variant="primary" size="md" onClick={handleAddDir}>
              添加
            </Button>
          </div>
        </Section>

        {/* Appearance */}
        <Section
          icon={<Palette size={18} />}
          title="外观"
          desc="主题与显示偏好。"
        >
          <div className="rounded-md border border-border bg-bg-elevated px-4 py-3 flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">深色主题</div>
              <div className="text-xs text-text-muted">
                EasyMusic 为深色模式设计，浅色主题计划中。
              </div>
            </div>
            <span className="text-xs px-2 py-0.5 rounded-full bg-accent/10 text-accent">
              已启用
            </span>
          </div>
          <div className="mt-2 rounded-md border border-border bg-bg-elevated px-4 py-3 flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">强调色</div>
              <div className="text-xs text-text-muted">Emerald (默认)</div>
            </div>
            <div className="w-6 h-6 rounded-full bg-accent" />
          </div>
        </Section>

        {/* Plugins */}
        <Section
          icon={<Plug size={18} />}
          title="插件"
          desc="扩展 EasyMusic 的功能。将插件放入 plugins/ 目录或从路径安装。"
        >
          {/* Plugin actions */}
          <div className="flex gap-2 mb-4">
            <Button variant="secondary" size="sm" onClick={handleReload}>
              <RefreshCw size={14} className="mr-1.5" />
              重新加载
            </Button>
          </div>

          {/* Plugin list */}
          <div className="space-y-2">
            {plugins.map((plugin) => (
              <PluginCard
                key={plugin.id}
                plugin={plugin}
                expanded={expandedPlugin === plugin.id}
                onToggleExpand={() =>
                  setExpandedPlugin(
                    expandedPlugin === plugin.id ? null : plugin.id,
                  )
                }
                onToggle={(enabled) => togglePlugin(plugin.id, enabled)}
                onUninstall={() => uninstallPlugin(plugin.id)}
              />
            ))}
            {plugins.length === 0 && (
              <p className="text-sm text-text-muted py-2">
                暂无已安装插件。将插件放入{" "}
                <code className="text-text">plugins/</code> 目录，或使用下方路径安装。
              </p>
            )}
          </div>

          {/* Install from path */}
          <div className="mt-4 flex gap-2 items-end">
            <div className="flex-1">
              <label className="block text-xs text-text-muted mb-1">
                从路径安装插件
              </label>
              <input
                type="text"
                placeholder="/path/to/my-plugin"
                value={installPath}
                onChange={(e) => setInstallPath(e.target.value)}
                className="w-full h-9 px-3 rounded-md bg-bg-elevated border border-border text-sm outline-none focus:border-accent/50"
              />
            </div>
            <Button variant="primary" size="md" onClick={handleInstallPlugin}>
              <FolderInput size={15} className="mr-1.5" />
              安装
            </Button>
          </div>
        </Section>

        <div className="text-xs text-text-muted border-t border-border pt-4 pb-8">
          EasyMusic 0.1.0 · Tauri + Next.js + Rust
        </div>
      </div>
    </div>
  );
}

/** A single plugin card with toggle, details, and uninstall. */
function PluginCard({
  plugin,
  expanded,
  onToggleExpand,
  onToggle,
  onUninstall,
}: {
  plugin: PluginInfo;
  expanded: boolean;
  onToggleExpand: () => void;
  onToggle: (enabled: boolean) => void;
  onUninstall: () => void;
}) {
  const isEnabled = plugin.status === "enabled";
  const isError = plugin.status === "error";

  return (
    <div className="rounded-md border border-border bg-bg-elevated overflow-hidden">
      <div className="flex items-center gap-3 px-4 py-3">
        <div
          className={`flex items-center justify-center w-9 h-9 rounded-md ${isError ? "bg-red-500/10 text-red-400" : "bg-bg-active text-text-secondary"}`}
        >
          <Plug size={16} />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">{plugin.name}</span>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-bg-active text-text-muted">
              v{plugin.version}
            </span>
            {plugin.author && (
              <span className="text-[10px] text-text-muted">by {plugin.author}</span>
            )}
          </div>
          <div className="text-xs text-text-muted truncate">
            {plugin.description || (isError ? plugin.error : "无描述")}
          </div>
        </div>

        {/* Toggle switch */}
        <button
          role="switch"
          aria-checked={isEnabled}
          aria-label={`切换 ${plugin.name}`}
          onClick={() => onToggle(!isEnabled)}
          disabled={isError}
          className={`relative w-10 h-6 rounded-full transition-colors ${
            isError
              ? "bg-red-500/30 opacity-50 cursor-not-allowed"
              : isEnabled
                ? "bg-accent"
                : "bg-border-strong"
          }`}
        >
          <span
            className={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
              isEnabled ? "translate-x-4" : ""
            }`}
          />
        </button>

        {/* Expand button */}
        <button
          onClick={onToggleExpand}
          className="text-text-muted hover:text-text p-1"
          aria-label={expanded ? "收起详情" : "展开详情"}
        >
          {expanded ? <ChevronUp size={15} /> : <ChevronDown size={15} />}
        </button>
      </div>

      {/* Expanded details */}
      {expanded && (
        <div className="border-t border-border px-4 py-3 space-y-2 bg-bg/50">
          <div className="text-xs text-text-muted">
            <span className="text-text-secondary">ID:</span>{" "}
            <code className="text-text">{plugin.id}</code>
          </div>

          {plugin.hooks.length > 0 && (
            <div className="text-xs">
              <span className="text-text-muted">钩子: </span>
              {plugin.hooks.map((h) => (
                <code
                  key={h}
                  className="inline-block mr-1 mb-1 px-1.5 py-0.5 rounded bg-accent/5 text-accent border border-accent/20"
                >
                  {h}
                </code>
              ))}
            </div>
          )}

          {plugin.permissions.length > 0 && (
            <div className="text-xs">
              <span className="text-text-muted inline-flex items-center gap-1">
                <Shield size={11} />
                权限:
              </span>{" "}
              {plugin.permissions.map((p) => (
                <code
                  key={p}
                  className="inline-block mr-1 mb-1 px-1.5 py-0.5 rounded bg-yellow-500/5 text-yellow-500 border border-yellow-500/20"
                >
                  {p}
                </code>
              ))}
            </div>
          )}

          {isError && plugin.error && (
            <div className="text-xs text-red-400 bg-red-500/5 border border-red-500/20 rounded px-2 py-1.5">
              {plugin.error}
            </div>
          )}

          <div className="flex justify-end pt-1">
            <button
              onClick={onUninstall}
              className="text-xs text-text-muted hover:text-danger inline-flex items-center gap-1"
            >
              <Trash2 size={12} />
              卸载
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function Section({
  icon,
  title,
  desc,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  desc: string;
  children: React.ReactNode;
}) {
  return (
    <section>
      <div className="flex items-center gap-2 mb-1">
        <span className="text-accent">{icon}</span>
        <h3 className="text-base font-semibold">{title}</h3>
      </div>
      <p className="text-xs text-text-muted mb-3">{desc}</p>
      {children}
    </section>
  );
}
