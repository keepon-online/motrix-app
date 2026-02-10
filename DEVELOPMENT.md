# Motrix v2.0.0 — 开发文档

> 全功能下载管理器，基于 **Tauri 2 + Vue 3 + Vite 5 + Aria2** 重构。

---

## 目录

- [技术架构](#技术架构)
- [目录结构](#目录结构)
- [环境要求](#环境要求)
- [快速开始](#快速开始)
- [构建与发布](#构建与发布)
- [后端架构 (Rust / Tauri)](#后端架构-rust--tauri)
- [前端架构 (Vue 3)](#前端架构-vue-3)
- [前后端通信](#前后端通信)
- [Aria2 引擎集成](#aria2-引擎集成)
- [配置管理](#配置管理)
- [功能清单](#功能清单)
- [依赖说明](#依赖说明)

---

## 技术架构

```
┌─────────────────────────────────────────────────────┐
│                    Motrix v2.0.0                    │
├────────────────────────┬────────────────────────────┤
│      前端 (WebView)    │      后端 (Rust)           │
│                        │                            │
│  Vue 3 + Composition   │  Tauri 2.0 Framework       │
│  Pinia 状态管理         │  24 个 Tauri Commands      │
│  Vue Router 4          │  8 个 Tauri Plugins        │
│  Element Plus UI       │  Aria2 JSON-RPC Client     │
│  Vite 5 构建           │  WebSocket 通信            │
│  TypeScript 5          │  系统托盘管理               │
│                        │                            │
├────────────────────────┼────────────────────────────┤
│    invoke() / emit()   │  Sidecar: aria2c binary    │
│    IPC 双向通信         │  RPC Port: 16800           │
└────────────────────────┴────────────────────────────┘
```

**与旧版对比：**

| 维度 | 旧版 (v1.x) | 新版 (v2.0) |
|------|------------|------------|
| 桌面框架 | Electron | Tauri 2.0 |
| 前端框架 | Vue 2 + Vuex | Vue 3 + Pinia |
| 构建工具 | Webpack | Vite 5 |
| UI 库 | Element UI | Element Plus |
| 后端语言 | Node.js | Rust |
| 打包体积 | ~120MB | ~15MB |
| 内存占用 | ~200MB | ~30MB |

---

## 目录结构

```
Motrix/
├── index.html                          # HTML 入口
├── package.json                        # NPM 配置 (前端依赖)
├── vite.config.ts                      # Vite 构建配置
├── tsconfig.json                       # TypeScript 配置
├── tsconfig.node.json                  # Node 端 TS 配置
│
├── src-tauri/                          # ===== Rust 后端 =====
│   ├── Cargo.toml                      # Rust 项目配置
│   ├── Cargo.lock                      # Rust 依赖锁定
│   ├── build.rs                        # 构建脚本
│   ├── tauri.conf.json                 # Tauri 应用配置
│   ├── capabilities/
│   │   └── default.json                # Tauri 2 权限声明
│   ├── binaries/
│   │   └── aria2c-{target_triple}      # aria2c sidecar 二进制
│   ├── icons/                          # 应用图标资源
│   │   ├── icon.png                    # 主图标
│   │   ├── 512x512.png                 # 高分辨率图标
│   │   ├── tray.png                    # 托盘图标
│   │   └── mo-tray-*.png              # 各平台托盘图标变体
│   └── src/
│       ├── main.rs                     # 应用入口 — 插件注册、命令注册
│       ├── lib.rs                      # 库入口 — 模块导出
│       ├── aria2.rs                    # Aria2 JSON-RPC over WebSocket 客户端
│       ├── commands.rs                 # 24 个 Tauri 命令定义
│       ├── config.rs                   # AppConfig 结构体与 aria2 参数转换
│       ├── error.rs                    # 统一错误类型 (7 种变体)
│       └── tray.rs                     # 系统托盘菜单与事件处理
│
├── src-vue/                            # ===== Vue 3 前端 =====
│   ├── main.ts                         # 应用入口 — Pinia/Router/i18n/ElementPlus
│   ├── App.vue                         # 根组件 — 布局框架
│   ├── assets/
│   │   └── logo.svg                    # Logo 资源
│   ├── styles/
│   │   └── index.scss                  # 全局样式
│   ├── types/
│   │   ├── index.ts                    # 核心类型定义 (9 个类型)
│   │   ├── auto-imports.d.ts           # 自动导入声明 (generated)
│   │   └── components.d.ts            # 组件声明 (generated)
│   ├── router/
│   │   └── index.ts                    # 路由配置 (5 个路由)
│   ├── stores/
│   │   ├── index.ts                    # Store 统一导出
│   │   ├── app.ts                      # 应用配置 Store (6 actions)
│   │   └── task.ts                     # 任务管理 Store (22 actions)
│   ├── composables/
│   │   ├── useTheme.ts                 # 主题管理 (auto/light/dark)
│   │   └── useAria2Events.ts           # Aria2 WebSocket 事件监听
│   ├── locales/
│   │   ├── en.ts                       # 英文
│   │   └── zh-CN.ts                    # 简体中文
│   ├── utils/
│   │   └── index.ts                    # 工具函数 (7 个)
│   ├── components/
│   │   ├── TitleBar.vue                # 自定义标题栏 (最小化/最大化/关闭)
│   │   ├── Sidebar.vue                 # 侧边栏导航 + 速度显示
│   │   ├── AddTaskDialog.vue           # 添加任务对话框 (URL/Torrent)
│   │   ├── DragDrop.vue                # 全局拖拽上传 (.torrent / URL)
│   │   ├── TaskItem.vue                # 单个任务条目 (进度/操作)
│   │   ├── TaskToolbar.vue             # 任务工具栏 (批量操作)
│   │   ├── TaskDetail.vue              # 任务详情抽屉面板
│   │   ├── TaskActivity.vue            # 速度图表 (Canvas 实时绘制)
│   │   ├── TaskFiles.vue               # 任务文件列表
│   │   ├── TaskPeers.vue               # BT Peers 表格
│   │   └── TaskTrackers.vue            # BT Tracker 列表
│   └── views/
│       ├── Tasks.vue                   # 任务列表页 (动态轮询/快捷键)
│       ├── Settings.vue                # 设置页 (6 个分类)
│       └── About.vue                   # 关于页
│
└── dist/                               # Vite 构建输出 (generated)
```

---

## 环境要求

| 工具 | 最低版本 | 说明 |
|------|---------|------|
| **Node.js** | 20.0.0+ | 前端构建 |
| **Rust** | 1.70+ | 后端编译 |
| **aria2c** | 1.36+ | 下载引擎 |

**系统依赖 (Linux)：**

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

---

## 快速开始

```bash
# 1. 安装前端依赖
npm install

# 2. 准备 aria2c sidecar（需要已安装 aria2c）
#    Tauri sidecar 命名规则：aria2c-{target_triple}
cp $(which aria2c) src-tauri/binaries/aria2c-$(rustc -vV | grep host | awk '{print $2}')

# 3. 启动开发模式
npm run tauri:dev
```

**其他命令：**

| 命令 | 说明 |
|------|------|
| `npm run dev` | 仅启动前端 Vite 开发服务器 (localhost:1420) |
| `npm run build` | TypeScript 检查 + Vite 前端构建 |
| `npm run tauri:dev` | 完整 Tauri 开发模式 (前端+后端+HMR) |
| `npm run tauri:build` | 生产构建，输出平台安装包 |
| `npm run lint` | ESLint 代码检查 |
| `npm run format` | Prettier 代码格式化 |

---

## 构建与发布

```bash
# 生产构建
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`：

| 平台 | 产物 |
|------|------|
| Linux | `.deb`, `.AppImage` |
| macOS | `.dmg`, `.app` |
| Windows | `.msi`, `.exe` |

---

## 后端架构 (Rust / Tauri)

### 模块说明

#### `main.rs` — 应用入口

注册 8 个 Tauri 插件和 24 个命令，初始化日志和 aria2 引擎。

```
Plugins: dialog, fs, notification, shell, store, process, os, clipboard-manager
```

#### `lib.rs` — 库入口

```rust
pub mod aria2;      // Aria2 引擎通信
pub mod commands;   // Tauri 命令
pub mod config;     // 配置管理
pub mod error;      // 错误处理
pub mod tray;       // 系统托盘
```

#### `aria2.rs` — Aria2 RPC 客户端

通过 WebSocket 与 aria2c 进程通信，实现 JSON-RPC 2.0 协议。

**核心结构：**

- `Aria2Client` — RPC 客户端，维护 WebSocket 连接
- `Aria2Event` / `Aria2EventType` — 事件类型定义
- `ARIA2_CLIENT` — 全局单例 (`RwLock<Option<Arc<Aria2Client>>>`)

**RPC 方法 (20 个)：**

| 方法 | 说明 |
|------|------|
| `addUri` | 添加 URI 下载 |
| `addTorrent` | 添加种子下载 |
| `pause` / `forcePause` | 暂停任务 (BT 需 force) |
| `unpause` | 恢复任务 |
| `remove` / `forceRemove` | 移除任务 |
| `tellStatus` | 获取单任务状态 |
| `tellActive` | 获取活跃任务 |
| `tellWaiting` | 获取等待任务 |
| `tellStopped` | 获取已停止任务 |
| `getGlobalStat` | 全局统计 |
| `changeGlobalOption` | 修改全局选项 |
| `changeOption` | 修改任务选项 |
| `getGlobalOption` / `getOption` | 获取选项 |
| `getVersion` | 获取版本 |
| `getPeers` | 获取 BT Peers |
| `saveSession` | 保存会话 |
| `pauseAll` / `forcePauseAll` / `unpauseAll` | 批量操作 |
| `removeDownloadResult` / `purgeDownloadResult` | 清理记录 |
| `multicall` | 批量 RPC |
| `shutdown` | 关闭引擎 (先保存会话) |

**事件处理 (6 种)：**

aria2 通过 WebSocket 推送通知，后端解析后通过 `app.emit("aria2-event", ...)` 转发给前端。

| 事件 | 触发时机 |
|------|---------|
| `DownloadStart` | 下载开始 |
| `DownloadPause` | 下载暂停 |
| `DownloadStop` | 下载停止 |
| `DownloadComplete` | 下载完成 |
| `DownloadError` | 下载出错 |
| `BtDownloadComplete` | BT 下载完成 |

#### `commands.rs` — Tauri 命令 (24 个)

所有命令均为 `async`，通过 `invoke()` 被前端调用。

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_app_config` | `app: AppHandle` | `AppConfig` | 加载应用配置 |
| `save_app_config` | `app, config` | `()` | 保存配置 |
| `add_uri` | `uris, options?` | `String` (gid) | 添加 URL 下载 |
| `add_torrent` | `torrent, options?` | `String` (gid) | 添加种子下载 |
| `pause_task` | `gid` | `String` | 暂停 |
| `resume_task` | `gid` | `String` | 恢复 |
| `remove_task` | `gid` | `String` | 移除 |
| `get_task_list` | `task_type` | `Value` | 任务列表 (active/waiting/stopped) |
| `get_task_info` | `gid` | `Value` | 单任务详情 |
| `get_global_stat` | — | `Value` | 全局统计 |
| `change_global_option` | `options` | `Value` | 修改全局选项 |
| `shutdown_engine` | — | `Value` | 关闭引擎 |
| `pause_all_tasks` | — | `Value` | 全部暂停 |
| `resume_all_tasks` | — | `Value` | 全部恢复 |
| `remove_task_record` | `gid` | `Value` | 移除记录 |
| `purge_task_records` | — | `Value` | 清除所有已停止记录 |
| `open_file` | `path` | `()` | 系统打开文件 |
| `show_in_folder` | `path` | `()` | 在文件管理器中显示 |
| `save_session` | — | `Value` | 保存 aria2 会话 |
| `force_pause_task` | `gid` | `String` | 强制暂停 (BT) |
| `force_remove_task` | `gid` | `String` | 强制移除 |
| `get_engine_version` | — | `Value` | 获取 aria2 版本 |
| `get_task_peers` | `gid` | `Value` | 获取 BT Peers |
| `change_task_option` | `gid, options` | `Value` | 修改任务选项 |

#### `config.rs` — 配置结构

`AppConfig` 含 29 个字段，分 6 个分组：

| 分组 | 字段 |
|------|------|
| 基本设置 | `locale`, `theme`, `download_dir`, `auto_start`, `start_hidden`, `hide_on_close`, `notify_on_complete`, `auto_clear_completed` |
| 下载设置 | `max_concurrent_downloads`, `max_connection_per_server`, `split`, `min_split_size`, `max_download_limit`, `max_upload_limit` |
| BT 设置 | `bt_listen_port`, `dht_listen_port`, `enable_upnp`, `seed_ratio`, `seed_time`, `bt_tracker`, `tracker_source` |
| 高级设置 | `user_agent`, `rpc_port`, `rpc_secret` |
| 代理设置 | `proxy_enabled`, `proxy_type`, `proxy_host`, `proxy_port`, `proxy_username`, `proxy_password` |

`to_aria2_args()` 方法将配置转换为 aria2c 命令行参数。

#### `error.rs` — 错误类型

```rust
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    Aria2Rpc(String),
    WebSocket(String),
    Config(String),
    Tauri(tauri::Error),
    Store(tauri_plugin_store::Error),
    Custom(String),
}
```

实现了 `Serialize` 以支持 Tauri 命令返回错误到前端。

#### `tray.rs` — 系统托盘

菜单项：Show Motrix / Pause All / Resume All / Quit

- 左键单击：显示主窗口
- 右键：显示菜单
- Quit：先调用 `aria2.shutdown()` 保存会话再退出

---

## 前端架构 (Vue 3)

### 入口 (`main.ts`)

初始化顺序：Pinia → Vue Router → vue-i18n → Element Plus → mount

### 路由 (`router/index.ts`)

| 路径 | 组件 | 说明 |
|------|------|------|
| `/` | → `/tasks/active` | 重定向 |
| `/tasks/:status` | `Tasks.vue` | active / waiting / stopped |
| `/settings` | `Settings.vue` | 设置页 |
| `/about` | `About.vue` | 关于页 |

### 状态管理 (Pinia Stores)

#### `useAppStore` — 应用配置

| API | 类型 | 说明 |
|-----|------|------|
| `config` | State | 当前配置 (AppConfig) |
| `isDark` | Getter | 是否暗色模式 |
| `locale` | Getter | 当前语言 |
| `downloadDir` | Getter | 下载目录 |
| `init()` | Action | 从后端加载配置 |
| `saveConfig(partial)` | Action | 保存配置并同步 aria2 引擎选项 |
| `setTheme(theme)` | Action | 设置主题 |
| `setLocale(locale)` | Action | 设置语言 |
| `setDownloadDir(dir)` | Action | 设置下载目录 |

**配置同步机制：** `saveConfig` 会自动检测下载相关选项的变更（如 `maxConcurrentDownloads`, `split` 等），并通过 `change_global_option` 命令实时同步到 aria2 引擎。

#### `useTaskStore` — 任务管理

| API | 类型 | 说明 |
|-----|------|------|
| `tasks` | State | 当前任务列表 |
| `currentListType` | State | 当前列表类型 |
| `selectedGids` | State | 已选中任务 GID |
| `currentTask` | State | 当前详情任务 |
| `detailVisible` | State | 详情面板是否可见 |
| `globalStat` | State | 全局统计 |
| `activeTasks` | Getter | 活跃任务 |
| `downloadSpeed` / `uploadSpeed` | Getter | 总速度 |
| `fetchTasks(type?)` | Action | 获取任务列表 |
| `addUri(uris, opts?)` | Action | 添加 URL 下载 |
| `addTorrent(data, opts?)` | Action | 添加种子下载 |
| `pauseTask(gid)` | Action | 暂停 (BT 自动用 forcePause) |
| `resumeTask(gid)` | Action | 恢复 |
| `removeTask(gid)` | Action | 移除 |
| `toggleTask(task)` | Action | 切换暂停/恢复 |
| `pauseAllTasks()` | Action | 全部暂停 |
| `resumeAllTasks()` | Action | 全部恢复 |
| `selectAllTasks()` | Action | 全选 |
| `pauseSelectedTasks()` | Action | 批量暂停 |
| `removeSelectedTasks()` | Action | 批量移除 |
| `purgeTaskRecords()` | Action | 清除所有已停止记录 |

### 组合式函数 (Composables)

#### `useTheme()`

基于 `@vueuse/core` 的 `useDark`，支持 auto / light / dark 三种模式。`auto` 模式跟随系统偏好。

#### `useAria2Events()`

监听 Tauri 事件 `aria2-event`，处理 6 种 aria2 通知：

- 下载开始/暂停/停止 → 刷新任务列表
- 下载完成/BT 完成 → 弹出成功通知 + 刷新列表
- 下载出错 → 弹出错误通知 + 刷新列表

### 组件说明

| 组件 | 功能 |
|------|------|
| `TitleBar.vue` | 自定义标题栏，支持窗口拖拽、最小化/最大化/关闭，`hideOnClose` 可配置关闭按钮行为 |
| `Sidebar.vue` | 侧边导航 (Downloads/Waiting/Completed)，底部显示实时上传/下载速度 |
| `AddTaskDialog.vue` | 添加任务对话框，支持 URL 和 Torrent 两种方式，打开时自动检测剪贴板 URL，支持高级选项 (User-Agent/Referer/Cookie/Authorization) |
| `DragDrop.vue` | 全局拖拽覆层，支持拖入 .torrent 文件或 URL 文本直接添加下载 |
| `TaskItem.vue` | 任务列表项，显示名称/进度条/速度/操作按钮 (暂停/恢复/打开文件/文件夹/复制链接/删除) |
| `TaskToolbar.vue` | 工具栏，支持全选/批量暂停/恢复/删除，无选中时显示 Pause All / Resume All |
| `TaskDetail.vue` | 右侧抽屉详情面板，显示进度/速度/大小/连接数/ETA/URL/保存路径/BT 信息 |
| `TaskActivity.vue` | Canvas 实时速度折线图，60 秒历史，1 秒刷新 (蓝色=下载, 绿色=上传) |
| `TaskFiles.vue` | 任务文件列表，显示文件名/大小/进度 |
| `TaskPeers.vue` | BT Peers 表格，显示 IP/客户端/下载速度/上传速度/是否 Seeder，3 秒刷新 |
| `TaskTrackers.vue` | BT Tracker 列表 |

### 工具函数 (`utils/index.ts`)

| 函数 | 说明 | 示例 |
|------|------|------|
| `formatBytes(bytes)` | 字节格式化 | `1536000` → `"1.46 MB"` |
| `formatSpeed(bps)` | 速度格式化 | `1048576` → `"1 MB/s"` |
| `formatDuration(seconds)` | 时间格式化 | `3661` → `"1:01:01"` |
| `calcRemainingTime(total, completed, speed)` | 计算剩余时间 | 返回秒数 |
| `calcProgress(total, completed)` | 计算进度百分比 | 返回 0-100 |
| `getTaskName(task)` | 获取任务名称 | 优先 BT 名称，否则从文件路径提取 |
| `isBtTask(task)` | 是否 BT 任务 | 检查 `bittorrent` 字段 |

### 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl/Cmd + A` | 全选任务 |
| `Escape` | 清除选中 / 关闭详情面板 |
| `Delete` | 删除选中任务 |

### 动态轮询策略

| 活跃任务数 | 轮询间隔 |
|-----------|---------|
| > 5 | 500ms |
| > 0 | 1000ms |
| 0 | 3000ms |

---

## 前后端通信

### IPC 通信

前端通过 `@tauri-apps/api/core` 的 `invoke()` 调用后端命令：

```typescript
// 前端调用
const tasks = await invoke<Task[]>('get_task_list', { taskType: 'active' })

// 后端定义
#[tauri::command]
pub async fn get_task_list(task_type: String) -> Result<Value> { ... }
```

### 事件通信

后端通过 `app.emit()` 向前端推送事件，前端通过 `listen()` 监听：

```
aria2c (WebSocket) → Rust 后端 (解析) → app.emit("aria2-event") → Vue 前端 (useAria2Events)
```

### Tauri 2 权限系统

`src-tauri/capabilities/default.json` 声明了所有必需权限，覆盖：

- 核心窗口操作 (show/hide/close/minimize/maximize/dragging...)
- 对话框 (open/save/message/ask/confirm)
- 文件系统 (read/write/exists/mkdir)
- 通知 (permission/notify)
- Shell (open/spawn/execute)
- 持久化存储、进程管理、系统信息、剪贴板

---

## Aria2 引擎集成

### 启动流程

```
Tauri App 启动
  → setup() 回调
    → aria2::init_engine()
      → 读取 Store 中的 config (rpcPort, rpcSecret)
      → 创建 session 文件和 DHT 文件
      → shell.sidecar("aria2c").args(...).spawn()
      → 等待 500ms
      → Aria2Client::new() — 建立 WebSocket 连接
      → 存入全局 ARIA2_CLIENT
```

### Aria2c 启动参数

```
--enable-rpc=true
--rpc-listen-port={port}
--rpc-secret={secret}
--rpc-listen-all=false
--rpc-allow-origin-all=true
--max-concurrent-downloads=10
--max-connection-per-server=16
--split=16
--min-split-size=1M
--enable-dht=true
--enable-dht6=true
--bt-enable-lpd=true
--follow-torrent=true
--check-certificate=false
--save-session={session_path}
--input-file={session_path}
--save-session-interval=10
--dht-file-path={dht_path}
--dht-file-path6={dht6_path}
```

### 会话持久化

- `aria2.session` — 任务会话文件，每 10 秒自动保存
- `dht.dat` / `dht6.dat` — DHT 路由表持久化
- 关闭时先调用 `saveSession()` 再 `shutdown()`
- 启动时通过 `--input-file` 恢复上次会话

### Sidecar 配置

`tauri.conf.json` 中的 `bundle.externalBin` 声明 sidecar：

```json
{ "externalBin": ["binaries/aria2c"] }
```

二进制文件命名规则：`aria2c-{target_triple}`，例如 `aria2c-x86_64-unknown-linux-gnu`。

---

## 配置管理

### 存储方式

使用 `tauri-plugin-store` 持久化到 `{app_data_dir}/config.json`，数据存储在 `config` 键下。

### 配置字段 (29 个)

```typescript
interface AppConfig {
  // 基本
  locale: string                    // 语言 (en, zh-CN)
  theme: 'auto' | 'light' | 'dark' // 主题
  downloadDir: string               // 下载目录
  autoStart: boolean                // 开机启动
  startHidden: boolean              // 启动时隐藏
  hideOnClose: boolean              // 关闭时最小化到托盘
  notifyOnComplete: boolean         // 下载完成通知
  autoClearCompleted: boolean       // 自动清理已完成任务

  // 下载
  maxConcurrentDownloads: number    // 最大并发数 (1-20, 默认 10)
  maxConnectionPerServer: number    // 单服务器最大连接数 (1-64, 默认 16)
  split: number                     // 文件分片数 (1-64, 默认 16)
  minSplitSize: string              // 最小分片大小 (默认 "1M")
  maxDownloadLimit: string          // 下载限速 (默认 "0" 无限)
  maxUploadLimit: string            // 上传限速 (默认 "0" 无限)

  // BT
  btListenPort: number              // BT 监听端口 (默认 21301)
  dhtListenPort: number             // DHT 监听端口 (默认 21302)
  enableUpnp: boolean               // 启用 UPnP
  seedRatio: number                 // 做种比率 (默认 1.0)
  seedTime: number                  // 做种时间 (分钟, 默认 60)
  btTracker: string                 // BT Tracker 列表
  trackerSource: string[]           // Tracker 源 URL 列表

  // 高级
  userAgent: string                 // User-Agent
  rpcPort: number                   // RPC 端口 (默认 16800)
  rpcSecret: string                 // RPC 密钥 (自动生成 UUID)

  // 代理
  proxyEnabled: boolean             // 启用代理
  proxyType: 'http' | 'https' | 'socks5'
  proxyHost: string
  proxyPort: number                 // 默认 1080
  proxyUsername: string
  proxyPassword: string
}
```

---

## 功能清单

### 核心下载功能

- [x] HTTP/HTTPS/FTP 下载
- [x] Magnet 链接下载
- [x] BitTorrent 种子下载
- [x] 多线程分片下载 (可配置 split 和连接数)
- [x] 下载限速 / 上传限速
- [x] 任务暂停 / 恢复 / 删除
- [x] BT 任务强制暂停 (forcePause)
- [x] 批量暂停 / 恢复 / 删除
- [x] 全部暂停 / 全部恢复
- [x] 会话持久化 (重启后恢复任务)
- [x] DHT 网络持久化

### 任务管理

- [x] 任务列表 (活跃 / 等待 / 已完成)
- [x] 任务详情 (进度/速度/大小/连接数/ETA)
- [x] 实时速度图表 (Canvas)
- [x] BT Peers 列表 (客户端检测)
- [x] BT Tracker 列表
- [x] 任务文件列表
- [x] 打开文件 / 在文件夹中显示
- [x] 复制下载链接
- [x] 清除已停止任务记录

### 用户界面

- [x] 自定义标题栏 (无原生装饰)
- [x] 侧边栏导航
- [x] 系统托盘 (左键显示/右键菜单)
- [x] 暗色模式 (auto/light/dark)
- [x] 拖拽添加 (.torrent 文件 / URL 文本)
- [x] 剪贴板自动检测 URL
- [x] 下载完成系统通知
- [x] 键盘快捷键 (全选/删除/ESC)
- [x] 动态轮询间隔 (根据活跃任务数调整)
- [x] 关闭窗口最小化到托盘 (可配置)

### 设置

- [x] 基本设置 (主题/语言/下载目录)
- [x] 下载设置 (并发数/连接数/分片数/限速)
- [x] BT 设置 (端口/UPnP/做种比率/Tracker)
- [x] 代理设置 (HTTP/HTTPS/SOCKS5)
- [x] 高级设置 (User-Agent/RPC 端口)
- [x] 添加任务高级选项 (Referer/Cookie/Authorization)
- [x] 配置实时同步到 aria2 引擎

### 国际化

- [x] 英文 (en)
- [x] 简体中文 (zh-CN)

---

## 依赖说明

### 前端依赖

| 包 | 版本 | 用途 |
|----|------|------|
| `vue` | ^3.5 | UI 框架 |
| `vue-router` | ^4.4 | 路由管理 |
| `pinia` | ^2.2 | 状态管理 |
| `vue-i18n` | ^10.0 | 国际化 |
| `element-plus` | ^2.8 | UI 组件库 |
| `@element-plus/icons-vue` | ^2.3 | 图标库 |
| `@vueuse/core` | ^11.0 | 组合式工具库 |
| `@tauri-apps/api` | ^2.0 | Tauri 前端 API |
| `@tauri-apps/plugin-*` | ^2.0 | Tauri 插件前端绑定 (8 个) |

### 开发依赖

| 包 | 版本 | 用途 |
|----|------|------|
| `vite` | ^5.4 | 构建工具 |
| `@vitejs/plugin-vue` | ^5.1 | Vue SFC 支持 |
| `typescript` | ^5.5 | 类型检查 |
| `vue-tsc` | ^2.1 | Vue TypeScript 编译器 |
| `sass` | ^1.77 | SCSS 预处理 |
| `unplugin-auto-import` | ^0.18 | 自动导入 |
| `unplugin-vue-components` | ^0.27 | 组件自动注册 |
| `@tauri-apps/cli` | ^2.0 | Tauri CLI |

### Rust 依赖

| Crate | 版本 | 用途 |
|-------|------|------|
| `tauri` | 2 | 桌面框架 |
| `tauri-plugin-*` | 2 | Tauri 插件 (8 个) |
| `tokio` | 1 (full) | 异步运行时 |
| `tokio-tungstenite` | 0.24 | WebSocket (连接 aria2) |
| `futures-util` | 0.3 | Future 工具 |
| `serde` / `serde_json` | 1 | JSON 序列化 |
| `thiserror` | 1 | 错误处理宏 |
| `tracing` / `tracing-subscriber` | 0.1/0.3 | 结构化日志 |
| `uuid` | 1 (v4) | UUID 生成 (RPC secret) |
| `dirs` | 5 | 系统目录路径 |
| `open` | 5 | 系统打开文件 |
| `url` | 2 | URL 解析 |
