# Deployment Guide (部署文档)

本指南介绍如何使用 Docker 和 Docker Compose 部署 SyncClipboard Server。

## 📦 使用 GitHub Container Registry (GHCR)

### 拉取镜像

```bash
# 拉取最新版本
docker pull ghcr.io/SyncClipboard/sync_clipboard_rs:latest

# 拉取特定版本
docker pull ghcr.io/SyncClipboard/sync_clipboard_rs:v0.1.0
```

### 运行容器

```bash
docker run -d \
  --name syncclipboard-server \
  -p 5033:5033 \
  -e SYNCCLIPBOARD_SERVER_HOST=0.0.0.0 \
  -e SYNCCLIPBOARD_SERVER_PORT=5033 \
  ghcr.io/SyncClipboard/sync_clipboard_rs:latest
```

### 使用 Docker Compose

```yaml
version: '3.8'
services:
  syncclipboard:
    image: ghcr.io/SyncClipboard/sync_clipboard_rs:latest
    container_name: syncclipboard-server
    ports:
      - "5033:5033"
    environment:
      - SYNCCLIPBOARD_SERVER_HOST=0.0.0.0
      - SYNCCLIPBOARD_SERVER_PORT=5033
    restart: unless-stopped
```

## 🔧 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `SYNCCLIPBOARD_SERVER_HOST` | `0.0.0.0` | 监听地址 |
| `SYNCCLIPBOARD_SERVER_PORT` | `5033` | 监听端口 |
| `SYNCCLIPBOARD_AUTH_TOKEN` | - | 可选的认证 Token |
| `SYNCCLIPBOARD_AUTH_ENCRYPT_PASSWORD` | - | 可选的 E2EE 密码 |

## 🚀 自动构建

每次推送 Git Tag（如 `v0.1.0`）时，GitHub Actions 会自动：
1. 构建 Docker 镜像（支持 amd64 和 arm64）
2. 推送到 GitHub Container Registry
3. 创建多个标签：
   - `v0.1.0` (精确版本)
   - `v0.1` (小版本)
   - `v0` (主版本)
   - `latest` (最新版)

## 🏗️ 本地构建

```bash
# 构建镜像
docker build -t syncclipboard-server .

# 运行
docker run -p 5033:5033 syncclipboard-server
```

## 📝 镜像说明

- **基础镜像**: Alpine Linux (最小化镜像大小)
- **架构支持**: amd64, arm64
- **镜像大小**: ~15MB (优化后)
- **包含组件**: 仅 Server 二进制文件（Desktop 不在 Docker 中）
