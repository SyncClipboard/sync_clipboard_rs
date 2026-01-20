use clipboard_core::config::Config;
use handlers::{AppState, get_clipboard, update_clipboard};
use file_handlers::{upload_file, get_download_file, head_file};
use std::sync::Arc;
use axum::{
    routing::{get, any, delete},
    Router,
};

pub mod handlers;
pub mod file_handlers;
mod auth;
use auth::auth_middleware;

mod db;
use db::Database;

mod webdav;
use webdav::WebDavRouter;

mod client_tracker;
use client_tracker::ClientTracker;

mod tracking_middleware;

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db = Database::new(&config.history.db_path, config.history.max_count)?;
    let tracker = Arc::new(ClientTracker::new());
    let state = AppState {
        db: Arc::new(db),
        notify: Arc::new(tokio::sync::Notify::new()),
        token: config.auth.token.clone(),
        tracker,
    };

    let mut router = Router::new()
        .route("/SyncClipboard.json", get(get_clipboard).put(update_clipboard))
        .route("/history", get(handlers::get_history_list))
        .route("/history/{id}", delete(handlers::delete_history).patch(handlers::pin_history))
        .route("/file/{filename}", get(get_download_file).put(upload_file).head(head_file))
        .route("/api/discovery", get(handlers::get_discovery_info))  // New: Discovery endpoint for cross-subnet scanning
        .route("/api/connected_devices", get(handlers::get_connected_devices));  // New: Get connected clients

    if config.server.wevdav_enabled {
        let upload_dir = "./uploads";
        std::fs::create_dir_all(upload_dir)?;
        let webdav = WebDavRouter::new(upload_dir);
        // Manual prefix stripping is more reliable with DavHandler than nest_service which can be tricky with non-standard methods
        let webdav_clone = webdav.clone();
        router = router.route("/webdav/{*path}", any(move |req: axum::extract::Request| async move {
            let mut req = req;
            let path = req.uri().path().to_string();
            if let Some(rest) = path.strip_prefix("/webdav") {
                let new_path = if rest.is_empty() { "/" } else { rest };
                let path_and_query = if let Some(query) = req.uri().query() {
                    format!("{}?{}", new_path, query)
                } else {
                    new_path.to_string()
                };
                *req.uri_mut() = axum::http::Uri::builder()
                    .path_and_query(path_and_query)
                    .build()
                    .unwrap();
            }
            webdav_clone.handle(req).await
        }));
        // Also handle the base "/webdav" path (without trailing slash/path)
        let webdav_clone = webdav.clone();
        router = router.route("/webdav", any(move |req: axum::extract::Request| async move {
             let mut req = req;
            let path_and_query = if let Some(query) = req.uri().query() {
                 format!("/?{}", query)
            } else {
                 "/".to_string()
            };
            *req.uri_mut() = axum::http::Uri::builder()
                .path_and_query(path_and_query)
                .build()
                .unwrap();
            webdav_clone.handle(req).await
        }));
        // Also handle the base "/webdav/" path (with trailing slash) explicitly
        let webdav_clone = webdav.clone();
        router = router.route("/webdav/", any(move |req: axum::extract::Request| async move {
             let mut req = req;
            let path_and_query = if let Some(query) = req.uri().query() {
                 format!("/?{}", query)
            } else {
                 "/".to_string()
            };
            *req.uri_mut() = axum::http::Uri::builder()
                .path_and_query(path_and_query)
                .build()
                .unwrap();
            webdav_clone.handle(req).await
        }));
    }
        
    let app = router
        .with_state(state.clone())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware))
        .layer(axum::middleware::from_fn_with_state(state.clone(), tracking_middleware::client_tracking_middleware));  // Track all clients BEFORE auth

    let addr_str = format!("{}:{}", config.server.host, config.server.port);
    let addr: std::net::SocketAddr = addr_str.parse()?; // Ensure host is IP or update config default to 0.0.0.0

    // ========== 混合发现服务初始化 ==========
    // 生成实例 ID（用于重启检测）
    let instance_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // 创建设备信息
    let my_device = clipboard_core::discovery::Device {
        id: config.general.device_id.clone(),
        name: config.general.device_name.clone(),
        ip: "0.0.0.0".to_string(),
        port: config.server.port,
        instance_id,
        capabilities: vec!["clipboard".to_string(), "file".to_string()],
    };
    
    // 启动混合发现服务（UDP Multicast + 缓存管理）
    match clipboard_core::discovery::DiscoveryService::new(my_device).await {
        Ok(discovery) => {
            let discovery_arc = std::sync::Arc::new(discovery);
            tokio::spawn(async move {
                tracing::info!("✓ Hybrid discovery service initialized successfully");
                discovery_arc.start().await;
            });
        }
        Err(e) => {
            tracing::error!("✗ Failed to initialize hybrid discovery service: {}", e);
            tracing::error!("  This means UDP multicast discovery will NOT work!");
            tracing::warn!("  Continuing with mDNS only for backward compatibility");
        }
    }
    // ========================================

    // mDNS Service Discovery (保留向后兼容)
    let mdns = mdns_sd::ServiceDaemon::new().map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;
    let service_type = "_syncclipboard._tcp.local.";
    let instance_name = &config.general.device_name; // Use configured device name
    let ip = "0.0.0.0"; // Allow mdns-sd to detect interfaces
    let host_name = format!("{}.local.", instance_name);
    let properties = [("version", "1.0")];

    let service_info = mdns_sd::ServiceInfo::new(
        service_type,
        instance_name,
        host_name.as_str(),
        ip,
        config.server.port,
        &properties[..],
    ).map_err(|e| format!("Invalid mDNS service info: {}", e))?;

    mdns.register(service_info).map_err(|e| format!("Failed to register mDNS service: {}", e))?;
    tracing::info!("mDNS registered: '{}' ({}) port {}", instance_name, service_type, config.server.port);

    if let Some(tls_config) = config.server.tls {
        tracing::info!("Starting HTTPS server on {}", addr);
        let config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
            tls_config.cert,
            tls_config.key,
        ).await?;

        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await?;
    } else {
        tracing::info!("Starting HTTP server on {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>()).await?;
    }
    Ok(())
}
