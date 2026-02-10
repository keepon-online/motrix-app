# Motrix

<p>
  <a href="https://motrix.app">
    <img src="./src-tauri/icons/icon.png" width="256" alt="Motrix App Icon" />
  </a>
</p>

## 一个全功能的下载管理器

[English](./README.md) | 简体中文

Motrix 是一个全功能的下载管理器，支持 HTTP、FTP、BitTorrent、Magnet 等协议。

基于 **Tauri 2 + Vue 3 + Vite + Aria2 + Rust** 构建。

## 功能特性

- 支持 HTTP / HTTPS / FTP / Magnet / BitTorrent 下载
- 多线程分片下载（可配置连接数和分片数）
- 下载 / 上传限速
- BitTorrent Peer Exchange、DHT、LPD 支持
- 任务会话持久化（重启后恢复任务）
- 系统托盘（全部暂停 / 全部恢复）
- 暗色模式（自动 / 浅色 / 深色）
- 拖拽 .torrent 文件或 URL 添加下载
- 剪贴板自动检测下载链接
- 下载完成系统通知
- 键盘快捷键（Ctrl+A 全选、Delete 删除、Escape 取消）
- 代理支持（HTTP / HTTPS / SOCKS5）
- 跨平台：Windows、macOS、Linux

## 安装

从 [GitHub Releases](https://github.com/agalwood/Motrix/releases) 下载安装。

## 开发

### 环境要求

| 工具 | 版本 |
|------|------|
| Node.js | 20.0.0+ |
| Rust | 1.70+ |
| aria2c | 1.36+ |

**Linux 系统依赖：**

```bash
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

### 开始开发

```bash
# 安装依赖
npm install

# 准备 aria2c sidecar
cp $(which aria2c) src-tauri/binaries/aria2c-$(rustc -vV | grep host | awk '{print $2}')

# 启动开发模式
npm run tauri:dev
```

### 命令

| 命令 | 说明 |
|------|------|
| `npm run dev` | 仅启动 Vite 开发服务器 |
| `npm run build` | TypeScript 检查 + Vite 构建 |
| `npm run tauri:dev` | 完整 Tauri 开发模式 |
| `npm run tauri:build` | 生产构建 |
| `npm run lint` | ESLint 代码检查 |
| `npm run format` | Prettier 格式化 |

详细架构文档请参阅 [DEVELOPMENT.md](./DEVELOPMENT.md)。

## 技术栈

| 组件 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | Vue 3 + Composition API |
| 状态管理 | Pinia |
| UI 组件库 | Element Plus |
| 构建工具 | Vite 5 |
| 后端 | Rust |
| 下载引擎 | aria2 (JSON-RPC over WebSocket) |
| 语言 | TypeScript + Rust |

## 开源协议

[MIT](LICENSE)

Copyright (c) 2018-present Dr_rOot
