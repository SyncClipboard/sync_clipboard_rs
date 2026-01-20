use axum::{
    extract::{State, ConnectInfo},
    http::Method,
};
use crate::handlers::AppState;
use std::net::SocketAddr;

/// 客户端追踪中间件
/// 记录所有请求的客户端信息
pub async fn client_tracking_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // 忽略 OPTIONS 请求 (CORS 预检请求通常不带自定义Header，会覆盖掉有效的设备名)
    if req.method() == Method::OPTIONS {
        return next.run(req).await;
    }
    // 提取客户端IP
    // 提取客户端IP
    // 优先使用 X-Forwarded-For (反向代理)，否则使用真实连接IP
    let client_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next().map(|s| s.trim().to_string()))
        .unwrap_or_else(|| addr.ip().to_string());

    // 提取设备名（从自定义头）
    let device_name = req
        .headers()
        .get("x-device-name")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 提取User-Agent
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 记录客户端
    state
        .tracker
        .record_client(client_ip, device_name, user_agent)
        .await;

    // 继续处理请求
    next.run(req).await
}
