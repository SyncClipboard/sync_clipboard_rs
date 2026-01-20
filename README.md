# SyncClipboard (Rust)

[![build](https://github.com/SyncClipboard/sync_clipboard_rs/actions/workflows/build.yml/badge.svg)](https://github.com/SyncClipboard/sync_clipboard_rs/actions/workflows/build.yml)

中文 | [English](docs/README_EN.md)

**SyncClipboard** 的 Rust 重写版本 (`sync_clipboard_rs`)。
本项目是 [Jeric-X/SyncClipboard](https://github.com/Jeric-X/SyncClipboard) 的 Rust 重构版本，旨在提供更高性能、更低资源占用的原生跨平台体验。

## ✨ 特性

-   **全平台支持**: 基于 [Tauri v2](https://v2.tauri.app/) 构建桌面端，通过 **UniFFI** 提供原生移动端 (Android/iOS) 绑定。
-   **高性能**: 核心逻辑采用 Rust 编写，资源占用极低。
-   **安全性**: 支持 **端到端加密 (E2EE)**，内置 Token 认证与 HTTPS/TLS 支持。
-   **高效率**: 采用 Hash 去重、**长轮询** 与 **流式传输**，实现大文件秒传与毫秒级同步延迟。
-   **富文本**: 支持 **HTML**、图片与纯文本的无损同步 (`clipboard-rs`)。
-   **易用性**: 支持 mDNS 局域网服务自动发现。
-   **自托管**: 内置独立 HTTP 服务器，支持文本、图片和文件同步。
-   **兼容性**: 完美兼容原版 [SyncClipboard](https://github.com/Jeric-X/SyncClipboard) 客户端。

## 🚀 快速开始

### 1️⃣ 下载安装

- **桌面端 (Windows / Linux / macOS)**: 请前往 [Releases](https://github.com/SyncClipboard/sync_clipboard_rs/releases) 下载对应平台的安装包 (`.exe`, `.deb`, `.appimage`, `.dmg`)。
- **移动端 (Android)**: 请前往 [magisk317/sync-clipboard-flutter](https://github.com/magisk317/sync-clipboard-flutter) 仓库下载最新适配的 APK。

### 2️⃣ 编译与运行 (开发者)
-   Rust (最新稳定版)
-   Node.js & pnpm (用于前端构建)
-   各平台构建工具 (如 Linux 需要 `libwebkit2gtk-4.0-dev` 等)

### 运行服务器
```bash
cd server
cargo run
```

### 运行客户端
```bash
cd desktop
cargo tauri dev
```

## ⚙️ 配置说明

可以通过 `config.toml` 文件或环境变量进行配置。环境变名前缀为 `SYNCCLIPBOARD_`。

| 配置项 | 环境变量 | 说明 | 默认值 |
| --- | --- | --- | --- |
| `server.port` | `SYNCCLIPBOARD_SERVER_PORT` | 服务器端口 | `5033` |
| `auth.token` | `SYNCCLIPBOARD_AUTH_TOKEN` | 访问令牌 (Bearer Token) | 无 |
| `auth.encrypt_password` | `SYNCCLIPBOARD_AUTH_ENCRYPT_PASSWORD` | E2EE 加密密码 (AES-256-GCM) | 无 |
| `server.tls.cert` | `SYNCCLIPBOARD_SERVER_TLS_CERT` | TLS 证书路径 (.pem) | 无 |
| `server.tls.key` | `SYNCCLIPBOARD_SERVER_TLS_KEY` | TLS 密钥路径 (.pem) | 无 |
| `history.max_count` | `SYNCCLIPBOARD_HISTORY_MAX_COUNT` | 保留的历史记录数量 | `100` |

### 启用端到端加密 (E2EE)
设置 `auth.encrypt_password` 后，所有上传的文本和 HTML 内容将在本地加密后传输，服务器仅存储密文。只有配置了相同密码的客户端才能解密查看。

### 启用 HTTPS
设置 `server.tls.cert` 和 `server.tls.key` 即可自动启用 HTTPS。

### 启用认证
设置 `auth.token` 后，所有请求必须携带 `Authorization: Bearer <token>` 标头。

## 📂 项目结构

| 目录 | 说明 |
| --- | --- |
| `clipboard_core` | 共享核心库 (配置, 数据模型, 计算逻辑, UniFFI 绑定) |
| `server` | 独立服务器 (Axum) |
| `desktop`| 跨平台客户端 (Tauri) |
| `cli` | 命令行工具 |

## 🔗 相关文档

- [开发日志](docs/development_log.md)
- [更新日志](docs/CHANGELOG.md)
- [部署文档](docs/deployment.md)

## 🙏 致谢

本项目深受 **[SyncClipboard](https://github.com/Jeric-X/SyncClipboard)** 的启发。

特别感谢原作者 [Jeric-X](https://github.com/Jeric-X) 在剪贴板同步领域的出色工作和开源贡献。本项目在协议设计和功能实现上大量参考了原项目的优秀设计。
