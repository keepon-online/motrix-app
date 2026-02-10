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
cp $(which aria2c) src-tauri/binaries/aria2c-$(rustc -vV | grep host | awk '{print $2}')

# Start development
npm run tauri:dev
```

### Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Start Vite dev server only |
| `npm run build` | TypeScript check + Vite build |
| `npm run tauri:dev` | Full Tauri development mode |
| `npm run tauri:build` | Production build |
| `npm run lint` | ESLint check |
| `npm run format` | Prettier format |

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
