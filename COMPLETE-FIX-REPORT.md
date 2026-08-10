# EasyMusic 代码问题修复 — 根任务总报告 (t_7edab2bf)

状态: 全部 8 项诊断问题已修复, 全链路验证通过, 已合并 main 并推送 GitHub (tag v0.1.1)
日期: 2026-08-10
根任务: t_7edab2bf (branch wt/t_7edab2bf, 提交 ef0b83e)
子任务产物: DIAGNOSTIC-REPORT.md (t_5161b11c) / FIX-REPORT.md (t_c6efd624) / RUN-NOTES.md (t_fbf6c8bd)

---

## 1. 问题清单与修复状态 (源自 t_5161b11c 诊断的 8 项)

| # | 问题 | 类别 | 状态 | 修复提交 |
|---|------|------|------|----------|
| 1 | cargo build --workspace 缺 glib-2.0 等系统库 | 构建失败·环境阻塞 | 已解决(环境方案) | bf24f39 + 本地 sysroot 虚拟化 |
| 2 | npm run start 与 output:export 互斥 | 运行时报错 | 已修复 | bf24f39 |
| 3 | PlaybackStatus current_track vs track_id 契约错位 | 运行时报错·Tauri 潜伏 | 已修复 | bf24f39 |
| 4 | RepeatMode Off/All/One vs off/all/one 大小写错位 | 运行时报错·Tauri 潜伏 | 已修复 | bf24f39 |
| 5 | plugin_commands.rs 死代码, 插件命令未注册, PluginManager 未 manage | 死代码/功能空壳 | 已修复 | bf24f39 |
| — | src-tauri 首次编译暴露的 2 处编译错误 (E0277/E0599) | 编译错误 | 已修复 | bf24f39 |
| 6 | scan 持锁: Mutex<LibraryManager> 让长扫描阻塞其他库读取 | 低优先级 | 已修复 | ef0b83e |
| 7 | 中英文混排: SettingsView 中文与其他英文 UI 不一致 | 低优先级 | 已修复 | ef0b83e |
| 8 | 双 lockfile 导致 Turbopack workspace-root 警告 | 低优先级 | 已修复 | ef0b83e |

## 2. 本任务 (根任务) 的收尾修复 — 提交 ef0b83e

- **Issue 6 (scan 持锁)**: src-tauri/src/commands.rs 的 `AppState.library` 由
  `Mutex<LibraryManager>` 改为 `RwLock<LibraryManager>` — 长扫描持有读锁,
  不再阻塞其他库读取命令; 仅 `library_open_db` 需要写锁。
- **Issue 7 (中英文混排)**: frontend SettingsView.tsx 全部中文 UI 字符串译为英文
  ("设置"→"Settings", "音乐库目录"→"Library Directories", "插件"→"Plugins" 等),
  与全站英文 UI 一致。
- **Issue 8 (双 lockfile 警告)**: frontend/next.config.ts 加
  `turbopack: { root: __dirname }` 固定项目根, 消除 "inferred your workspace
  root" 警告。
- **版本升级 0.1.0 → 0.1.1**: Cargo.toml / src-tauri/tauri.conf.json /
  package.json / frontend/package.json / SettingsView 页脚同步。

## 3. 全链路验证 (本任务实测, 2026-08-10)

| 验证项 | 命令 | 结果 |
|--------|------|------|
| Rust 全量构建 (含 Tauri 壳) | cargo build --workspace (sysroot env) | exit 0, 1m13s 增量 |
| Rust 全量测试 | cargo test --workspace | 39/39 (27 core + 12 sdk) |
| 前端干净构建 | npm run build (rm -rf .next out) | exit 0, 4 静态页 |
| 静态托管 | npm run start / npx serve out | HTTP 200 |
| 设置页 UI | 真实浏览器加载 /settings 视图 | 全英文渲染, 0 JS 错误 |
| 运行日志 | browser console | 0 console messages / 0 errors |

子任务已验证项 (继承): cargo clean && cargo build --workspace 从零 8m30s exit 0;
Tauri GUI 在 Xvfb+proot 下真实启动渲染 (截图证据见 RUN-NOTES.md); 5 视图浏览器
冒烟 0 JS 错误; cargo fmt --all 干净; npm run lint 0 错误。

## 4. 环境说明 (容器无 root)

- 系统库依赖通过本地 sysroot 虚拟化解决:
  `/opt/data/tauri-sysroot/root` (393+ .deb 闭包, 924 MB), 构建时 export
  PKG_CONFIG_PATH / PKG_CONFIG_SYSROOT_DIR / CFLAGS / LIBRARY_PATH /
  LD_LIBRARY_PATH (见 FIX-REPORT.md §2)。
- 无头运行 Tauri: `scripts/run-tauri-headless.sh` (Xvfb + proot 绑定 sysroot,
  见 RUN-NOTES.md)。正常桌面环境直接 `npm run tauri:dev` 即可。
- CI (GitHub Actions) 不受影响 — ci.yml 用 apt 安装系统依赖。

## 5. 交付

- main 分支已合并全部修复, 推送 https://github.com/ymslucky/EasyMusic
- 版本 tag: v0.1.1 (含全部修复的 0.1.1 构建)
- 全部 8 项诊断问题闭环, 构建/运行/测试全绿。
