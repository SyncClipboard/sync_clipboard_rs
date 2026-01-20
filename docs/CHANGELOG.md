# 更新日志 (Changelog)

本项目的所有重要更改都将记录在此文件中。

## [0.1.0-alpha] - 2026-01-16

### 新增功能 (Added)
- **核心 (Core)**: 完成逻辑层的 Rust 重写 (`clipboard_core`)。
- **服务端 (Server)**: 基于 `axum` 的高性能 HTTP 服务器。
- **客户端 (Desktop)**: 基于 Tauri v2 的跨平台客户端架构。
- **安全性**:
    - 端到端加密 (E2EE) (AES-256-GCM)。
    - Token 访问认证。
    - HTTPS/TLS 支持。
- **同步功能**:
    - 实时剪贴板同步 (支持文本、图片、HTML)。
    - 大文件流式上传/下载。
    - 基于 Hash 的文件与图片去重。
    - 智能长轮询机制 (延迟 <1s)。
- **服务发现**: mDNS 局域网自动发现 (`_syncclipboard._tcp`)。
- **移动端**: Android (Kotlin) 和 iOS (Swift) 的 UniFFI 绑定支持。

### 修复 (Fixed)
- 替换 `arboard` 为 `clipboard-rs` 以支持 HTML 富文本读取。
- 解决了流式传输中的大文件内存占用问题。
