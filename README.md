# Motrix

<p>
  <a href="https://motrix.app">
    <img src="./src-tauri/icons/icon.png" width="256" alt="Motrix App Icon" />
  </a>
</p>

## A full-featured download manager

English | [简体中文](./README-CN.md)

Motrix is a full-featured download manager that supports downloading HTTP, FTP, BitTorrent, Magnet, etc.

Built with **Tauri 2 + Vue 3 + Vite + Aria2 + Rust**.

## Features

- HTTP / HTTPS / FTP / Magnet / BitTorrent download support
- Multi-threaded split downloading (configurable connections & splits)
- Download / upload speed limiting
- BitTorrent peer exchange, DHT, LPD support
- Task session persistence across restarts
- System tray with Pause All / Resume All
- Dark mode (auto / light / dark)
- Drag & drop .torrent files or URLs
- Clipboard auto-detection
- Download complete system notification
- Keyboard shortcuts (Ctrl+A, Delete, Escape)
- Proxy support (HTTP / HTTPS / SOCKS5)
- Cross-platform: Windows, macOS, Linux

## Installation

Download from [GitHub Releases](https://github.com/agalwood/Motrix/releases) and install.

## Development

### Prerequisites

| Tool | Version |
|------|---------|
| Node.js | 20.0.0+ |
| Rust | 1.70+ |
| aria2c | 1.36+ |

**Linux system dependencies:**

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

### Setup

```bash
# Install dependencies
npm install

# Prepare aria2c sidecar
npm run sidecar:prepare

# Start development
npm run tauri:dev
```

### Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server only |
| `npm run build` | TypeScript check + Vite build |
| `npm run sidecar:prepare` | Prepare the aria2c sidecar declared in `sidecar-manifest.json` for the current target |
| `npm run sidecar:check` | Validate the prepared sidecar against the manifest rules |
| `npm run tauri:dev` | Full Tauri development mode |
| `npm run tauri:build` | Production build |
| `npm run lint` | ESLint check |
| `npm run format` | Prettier format |

Aria2 sidecars are managed by `sidecar-manifest.json`. Default builds download
repository-managed prebuilt sidecars from the `aria2-sidecar-<version>` GitHub
release tag and validate the result against the manifest rules.

Emergency regeneration is explicit. Use `MOTRIX_SIDECAR_REGENERATE=1` to switch
to the target's fallback strategy. Linux and macOS fall back to source builds;
Windows falls back to the upstream official aria2 archive.

Publish or refresh repository-managed sidecars with the
`Publish Aria2 Sidecars` GitHub Actions workflow. It also emits an updated
`sidecar-manifest.json` artifact with refreshed prebuilt sidecar SHA256 values.

See [DEVELOPMENT.md](./DEVELOPMENT.md) for detailed architecture documentation.

## Tech Stack

| Component | Technology |
|-----------|------------|
| Desktop Framework | Tauri 2.0 |
| Frontend | Vue 3 + Composition API |
| State Management | Pinia |
| UI Library | Element Plus |
| Build Tool | Vite 5 |
| Backend | Rust |
| Download Engine | aria2 (JSON-RPC over WebSocket) |
| Language | TypeScript + Rust |

## License

[MIT](LICENSE)

Copyright (c) 2018-present Dr_rOot
