# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Motrix v2.0 — A full-featured download manager (HTTP/FTP/BT/Magnet) built with **Tauri 2 + Vue 3 + Vite 5 + Aria2 + Rust**. This is a v2 rewrite from Electron/Vue 2 to Tauri/Vue 3.

## Build Commands

```bash
npm install              # Install frontend dependencies
npm run dev              # Vite dev server only (localhost:1420)
npm run build            # TypeScript check (vue-tsc --noEmit) + Vite production build
npm run tauri:dev        # Full Tauri dev mode (frontend + Rust backend + HMR)
npm run tauri:build      # Production build → installers in src-tauri/target/release/bundle/
npm run lint             # ESLint
npm run format           # Prettier (src-vue/ only)
```

**Prerequisites**: Node.js >= 20, Rust >= 1.70, aria2c >= 1.36

**Sidecar setup**: aria2c binary must be at `src-tauri/binaries/motrix-aria2c-{target_triple}` (e.g., `motrix-aria2c-x86_64-unknown-linux-gnu`).

**Linux deps**: `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`

## Architecture

```
Frontend (Vue 3 WebView)  ←→  Tauri IPC (invoke/emit)  ←→  Backend (Rust)
       ↓                                                        ↓
  Pinia stores                                         Aria2Client (WebSocket JSON-RPC 2.0)
  Element Plus UI                                               ↓
  vue-i18n (en, zh-CN)                                    aria2c sidecar process
```

### Backend (`src-tauri/src/`)

- **`main.rs`** — Entry: registers 15 Tauri plugins, ~45 invoke commands, tray setup, aria2 engine init, single-instance, CLI arg parsing
- **`lib.rs`** — Module exports: `aria2`, `cli`, `commands`, `config`, `error`, `power`, `tray`
- **`aria2/`** — Engine lifecycle with process watchdog (auto-restart, max 3 retries, exponential backoff), WebSocket RPC client (~25 methods), event forwarding
- **`commands.rs`** — ~45 `#[tauri::command]` async functions for task CRUD, config, file ops, engine management, tracker sync, factory reset
- **`config.rs`** — `AppConfig` with ~50 fields, `#[serde(rename_all = "camelCase")]` for seamless JSON interop, `to_aria2_args()` converts to CLI args
- **`tray.rs`** — System tray with i18n labels
- **`cli.rs`** — URL/torrent/metalink detection from CLI args, `thunder://` base64 decoding
- **`power.rs`** — Platform-specific sleep prevention (macOS IOKit, Windows SetThreadExecutionState, Linux D-Bus)

### Frontend (`src-vue/`)

- **Stores** (`stores/`): `useAppStore` (config with 500ms debounced save, auto-sync to aria2), `useTaskStore` (task CRUD, batch ops, BT auto-forcePause)
- **Composables** (`composables/`): `useTheme`, `useAria2Events` (6 aria2 event types), `useConnectionStatus` (shared connection state), `useUpdater`
- **Path alias**: `@` → `src-vue/`
- **Auto-imports**: `unplugin-auto-import` + `unplugin-vue-components` (Element Plus)
- **App version**: Injected as `__APP_VERSION__` via Vite `define`

### Frontend-Backend Communication

- **IPC**: `invoke('command_name', { args })` from `@tauri-apps/api/core` → Rust `#[tauri::command]`
- **Events**: Backend emits `aria2-event` (6 download events), `aria2-connection` (connection state), `aria2-ready` (engine ready), `open-urls` (CLI/deep-link URLs)
- **Flow**: `aria2c → WebSocket notification → Rust parse → app.emit() → Vue listen()`

### Key Patterns

- **Config persistence**: `tauri-plugin-store` → `config.json` in app data directory
- **Aria2 process**: Sidecar via `tauri-plugin-shell`, sensitive data (RPC secret, proxy password) written to `aria2.conf` file not CLI args
- **Dynamic polling**: Active task count determines interval (500ms >5 tasks, 1s >0, 3s idle)
- **Window behavior**: `hideOnClose` configurable — close button hides to tray instead of quitting

### Tauri Plugins (15)

`single-instance`, `deep-link`, `autostart`, `dialog`, `fs`, `notification`, `shell`, `store`, `process`, `os`, `clipboard-manager`, `updater`, `window-state`

## TypeScript Configuration

Strict mode with `noUnusedLocals` and `noUnusedParameters` enabled. Unused variables must be removed or consumed — underscore prefix does NOT suppress errors.

## CI

Single workflow (`.github/workflows/build.yml`): builds on tag push (`v*`) or manual dispatch. Builds aria2 v1.37.0 from source on Linux/macOS. Three parallel platform jobs. Uses `tauri-apps/tauri-action` to create GitHub draft releases.

## Language

The primary development language is Chinese (codebase docs, commit messages, DEVELOPMENT.md are in Chinese). UI supports English and Simplified Chinese via vue-i18n. Respond in the user's preferred language.
