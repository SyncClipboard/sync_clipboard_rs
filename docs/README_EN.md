# SyncClipboard (Rust)

[中文](../README.md) | English

A Rust rewrite of **SyncClipboard** (`sync_clipboard_rs`).
Designed to provide a high-performance, lightweight, and native cross-platform (Windows, Linux, macOS, Android, iOS) clipboard synchronization service.

## ✨ Features

-   **Cross-Platform**: Built with [Tauri v2](https://v2.tauri.app/), covering desktop and mobile.
-   **High Performance**: Core logic written in Rust for minimal resource usage.
-   **Secure**: Token authentication and HTTPS/TLS support built-in.
-   **Efficient**: Hash-based deduplication and Long Polling for low-latency synchronization.
-   **User-Friendly**: Built-in mDNS service discovery for easy LAN setup.
-   **Self-Hosted**: Built-in standalone HTTP server supporting text, image, and file synchronization.
-   **Compatibility**: Fully compatible with the original [SyncClipboard](https://github.com/Jeric-X/SyncClipboard) clients.

## 🚀 Getting Started

### Prerequisites
-   Rust (Latest Stable)
-   Node.js & pnpm (for frontend build)
-   Platform-specific build tools (e.g., `libwebkit2gtk-4.0-dev` for Linux)

### Run Server
```bash
cd server
cargo run
```

### Run Client
```bash
cd desktop
cargo tauri dev
```

## ⚙️ Configuration

Configuration can be set via `config.toml` or environment variables (prefixed with `SYNCCLIPBOARD_`).

| Config Key | Environment Variable | Description | Default |
| --- | --- | --- | --- |
| `server.port` | `SYNCCLIPBOARD_SERVER_PORT` | Server Port | `5033` |
| `auth.token` | `SYNCCLIPBOARD_AUTH_TOKEN` | Access Token (Bearer) | None |
| `server.tls.cert` | `SYNCCLIPBOARD_SERVER_TLS_CERT` | TLS Certificate Path | None |
| `server.tls.key` | `SYNCCLIPBOARD_SERVER_TLS_KEY` | TLS Key Path | None |
| `history.max_count` | `SYNCCLIPBOARD_HISTORY_MAX_COUNT` | Max History Items | `100` |

### Enable HTTPS
Set `server.tls.cert` and `server.tls.key` to enable HTTPS automatically.

### Enable Authentication
Set `auth.token` to require `Authorization: Bearer <token>` header for all requests.

## 📂 Project Structure

| Directory | Description |
| --- | --- |
| `core` | Shared Core Library (Config, Models) |
| `server` | Standalone Server (Axum) |
| `desktop`| Cross-Platform Client (Tauri) |
| `cli` | Command Line Interface |

## 🔗 Documentation

- [Rewrite Plan](rewrite_plan.md)
- [Progress](progress.md)
