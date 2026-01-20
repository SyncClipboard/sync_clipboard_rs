# 开发日志 (Development Log)

本文档合并了原有的「重写评估计划」与「实施进度表」，记录了项目的从零重构历程。

## 1. 项目起源与目标
**原项目**: [SyncClipboard](https://github.com/Jeric-X/SyncClipboard) (C# / ASP.NET Core)
**新项目**: `sync_clipboard_rs` (Rust Rewrite)
**目标**: 实现高性能、跨平台、低资源占用，且完美兼容原版协议的剪贴板同步服务。

## 2. 架构回顾
项目采用了 Cargo Workspace 结构，实现了模块化开发：
- `clipboard_core` (Lib): 核心逻辑库。包含配置解析、数据模型、加解密 (AES-256-GCM)、同步算法。
- `server` (Bin): 基于 Axum 的高性能 HTTP 服务器。支持 HTTPS、Token 认证、长轮询、WebDAV。
- `desktop` (Bin): 基于 Tauri v2 的桌面/移动端通用客户端。支持 Windows/Linux/macOS 及 Android/iOS (通过 UniFFI)。
- `cli` (Bin): 命令行测试工具。

## 3. 实施历程 (Completed Milestones)

### 阶段一：基础设施搭建
- [x] 初始化 Cargo Workspace (core, server, cli, desktop)
- [x] 实现 Core 基础逻辑 (配置, 剪贴板模型)
- [x] 实现 Server 基础 API (GET/PUT SyncClipboard.json)
- [x] 建立 Docker 构建流程 (`Dockerfile`)

### 阶段二：核心功能复刻
- [x] **本地剪贴板集成**: 使用 `clipboard-rs` 替代 `arboard` 以支持 HTML。
- [x] **持久化存储**: 集成 SQLite (`rusqlite`) 存储历史记录。
- [x] **WebDAV 支持**: 集成 `dav-server-rs` 提供文件访问。
- [x] **基础鉴权**: 实现 HTTP Basic Auth / Header Token 验证。

### 阶段三：性能与安全增强 (差异化优势)
- [x] **Hash 去重**: 实现文件/图片的 SHA256 去重与秒传检测。
- [x] **长轮询 (Long Polling)**: 实现 `<1s` 延迟的即时同步，替代低效轮询。
- [x] **端到端加密 (E2EE)**: 引入 AES-256-GCM，确保服务器无法窃听内容。
- [x] **HTTPS/TLS**: 支持服务端配置证书自动启用 SSL。
- [x] **自动清理**: 实现历史记录数量限制 (`history.max_count`)。
- [x] **mDNS 服务发现**: 实现局域网服务自动广播。

### 阶段四：高级特性与适配
- [x] **大文件流式传输**: 优化内存占用，支持 GB 级文件同步。
- [x] **富文本支持**: 完整支持 HTML/RTF 内容同步。
- [x] **移动端适配**: 
    - 采用 **Library Mode** 架构。
    - 使用 **UniFFI** 生成 Kotlin/Swift 原生绑定。

## 4. 技术栈总结
- **语言**: Rust 2021
- **Web 框架**: Axum 0.7
- **异步运行时**: Tokio 1.0
- **数据库**: SQLite (Rusqlite)
- **GUI**: Tauri v2
- **移动端桥接**: UniFFI
