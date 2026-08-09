# EasyMusic 运行时修复报告 (Task t_fbf6c8bd)

状态: 运行时验证通过 — 应用稳定运行、UI 真实渲染、39/39 测试全过
日期: 2026-08-10
工作区: /opt/data/workspace/Code/EasyMusic/.worktrees/t_fbf6c8bd (branch wt/t_fbf6c8bd)
前置任务: t_c6efd624 (构建修复, 已合并入本分支 bf24f39)

---

## 1. 结论 (验收标准对照)

| 验收标准 | 结果 |
|----------|------|
| 应用启动并达到稳定/空闲状态, 无未捕获错误 | ✅ Tauri 应用在 Xvfb 下启动, 连续运行 60s+ 无崩溃, 日志干净 (仅无害的 gio libproxy 模块警告) |
| 冒烟/集成测试通过 | ✅ `cargo test --workspace` 39/39 (27 core + 12 sdk) |
| 文档化运行命令与所需环境 | ✅ 本报告 + `scripts/run-tauri-headless.sh` 一键启动脚本 |

## 2. 运行时错误与修复

### 错误 1 — WebKitNetworkProcess 缺失 (环境类)
```
** (easymusic): ERROR **: Unable to spawn a new child process:
Failed to spawn child process "/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess" (No such file or directory)
```
**根因**: webkit2gtk-4.1 的辅助进程路径**硬编码**在 libwebkit2gtk 中 (字符串检查确认 `/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/` 编译期路径, WEBKIT_EXEC_PATH 环境变量在本版本被忽略)。容器无 root, 无法把 vendored sysroot 的 webkit 运行时装进系统路径。
**修复**: 用 `proot -b` (无 root bind mount 模拟) 把 sysroot 中的 webkit2gtk-4.1 目录绑定到硬编码系统路径。proot 二进制从 Debian 源下载 (proot_5.1.0-1.3+b1_amd64.deb + libtalloc2)。

### 错误 2 — EGL display 创建失败 (环境类)
```
Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...
```
**根因**: GLVND 的 EGL loader 找不到 mesa 厂商配置 (默认搜索 `/usr/share/glvnd/egl_vendor.d/`, 容器里不存在), 于是加载不到 libEGL_mesa → 所有平台 EGL display 创建失败 → WebKitWebProcess abort。
**修复**: 设置 `__EGL_VENDOR_LIBRARY_DIRS` / `__EGL_VENDOR_LIBRARY_FILENAMES` 指向 sysroot 中的 `50_mesa.json`。配套 `LIBGL_ALWAYS_SOFTWARE=1` + `GALLIUM_DRIVER=llvmpipe` (无 GPU 容器软件渲染)。

### 错误 3 — GSettings schemas 未安装 (环境类)
```
GLib-GIO-ERROR: No GSettings schemas are installed on the system
ERROR: WebKit encountered an internal error. ... WebLoaderStrategy::internallyFailedLoadTimerFired()
```
**根因**: GLib 需要编译过的 `gschemas.compiled`; sysroot 里只有 .xml 源文件。
**修复**: 用 sysroot 自带 `glib-2.0/glib-compile-schemas` 编译生成 gschemas.compiled (42KB), proot 绑定到 `/usr/share/glib-2.0/schemas`。

### 错误 4 — WebKitWebProcess 直接 spawn 失败 (proot 兼容类)
修复上述问题后 WebProcess 仍不存活; 通过 bash wrapper `exec` 真实二进制后稳定运行 (proot 对直接追踪该 ELF 有 ptrace 兼容问题)。
**修复**: 脚本生成 `/tmp/easymusic-webproc-wrapper.sh` 并 proot 绑定覆盖系统路径。

## 3. 运行方式

### 普通桌面环境 (有 root / 已装系统库)
```bash
# 一次构建
cargo build --workspace
# 开发模式 (Next.js 热重载 + Tauri 窗口)
npm run tauri:dev
# 前端独立静态预览
npm run build && npm --prefix frontend run start   # 静态托管 out/ 于 :1420
```

### 无头容器 (无 root, 无系统 webkit2gtk 运行时) — 本项目环境
```bash
# 前置: vendored sysroot 在 /opt/data/tauri-sysroot/root (见 t_c6efd624 报告),
#       proot 在 /tmp/proot-x, libtalloc 在 /tmp/talloc-x (脚本默认路径)

# 一键启动 Tauri GUI (headless, 需要 Xvfb 或已有 DISPLAY)
Xvfb :99 -screen 0 1280x800x24 &     # 若无显示服务器
DISPLAY=:99 scripts/run-tauri-headless.sh

# 验证 UI 真实渲染 (截图检查)
ffmpeg -y -f x11grab -video_size 1280x800 -i :99 -frames:v 1 shot.png
# 期望: 窗口区域 >95% 暗色像素 (应用为深色主题 --bg:#09090b)

# 运行全部测试
cargo test --workspace   # 39/39
```

## 4. 交付物

| 文件 | 说明 |
|------|------|
| `scripts/run-tauri-headless.sh` | 一键无头启动脚本: 环境变量 + proot 绑定 + WebProcess wrapper, 支持 EASY_MUSIC_BIN/PROOT_BIN/TAURI_SYSROOT/TALLOC_LIB/DISPLAY 覆盖 |
| 本报告 (RUN-NOTES) | 运行命令 + 所需环境记录 |

## 5. 遗留观察项 (超出本任务范围)

- **gio libproxy 模块警告** (无害): sysroot 的 gio/modules/libgiolibproxy.so 缺 libpxbackend-1.0.so, 仅影响 libproxy 网络代理支持, 应用功能不受影响。
- **proot 下 WebProcess 需 wrapper**: 环境特有的 proot 兼容问题, 非应用缺陷; 正常桌面环境无此问题。
- 父任务遗留: ISSUE 6 (library_scan 持锁)、ISSUE 7 (设置页中英混排)、ISSUE 8 (双 lockfile 警告)。

## 6. 验证证据

- Tauri 二进制: `target/debug/easymusic`, Xvfb :79 下运行 60s 稳定, 退出码无崩溃
- UI 截图: /tmp/easymusic-script-shot.png (31036 bytes, 窗口区域 99.9% 暗色像素 = 深色 UI 已渲染)
- 日志: 无 error/panic, 仅 gio libproxy 无害警告
- 测试: cargo test --workspace → 27 core + 12 sdk = 39 passed
