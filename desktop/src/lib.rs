use clipboard_core::config::Config;
use rusqlite::Connection;
use serde::Serialize;
use std::net::TcpListener;
use tauri::Manager;

#[derive(Serialize)]
struct HistoryItem {
    id: i64,
    r#type: String,
    content: Option<String>,
    html: Option<String>,
    file: Option<String>,
    device: Option<String>,
    timestamp: String,
    pinned: bool,
}

#[tauri::command]
fn get_config() -> Config {
    // Load config or return default (panics if fails generally, but acceptable for dev)
    Config::new().unwrap_or_else(|_| {
        Config::new().expect("Failed to initialize config")
    })
}

/// 检查端口是否可用
#[tauri::command]
fn check_port_available(port: u16) -> Result<bool, String> {
    match TcpListener::bind(("0.0.0.0", port)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 查找从指定端口开始的第一个可用端口
#[tauri::command]
fn find_available_port(start_port: u16) -> Result<u16, String> {
    for port in start_port..=65535 {
        if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)) {
            drop(listener);
            return Ok(port);
        }
    }
    Err("No available ports found".to_string())
}

#[tauri::command]
fn save_config(config: Config) -> Result<(), String> {
    // 检查服务器端口是否可用
    if config.server.enabled {
        match TcpListener::bind(("0.0.0.0", config.server.port)) {
            Ok(_) => {
                // 端口可用，继续保存
            }
            Err(_) => {
                // 端口被占用，查找下一个可用端口
                let suggested_port = find_available_port(config.server.port + 1)
                    .unwrap_or(config.server.port + 1);
                return Err(format!(
                    "端口 {} 已被占用，建议使用端口 {}",
                    config.server.port, suggested_port
                ));
            }
        }
    }
    
    // Use the new persistence implementation
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_history() -> Result<Vec<HistoryItem>, String> {
    let config = Config::new().map_err(|e| e.to_string())?;
    let conn = Connection::open(&config.history.db_path).map_err(|e| e.to_string())?;
    
    // Sort by pinned DESC (pinned first), then by id DESC (newest first)
    let mut stmt = conn.prepare("SELECT id, type, content, html, file, device, timestamp, pinned FROM history ORDER BY pinned DESC, id DESC LIMIT 50").map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(HistoryItem {
            id: row.get(0)?,
            r#type: row.get(1)?,
            content: row.get(2)?,
            html: row.get(3).unwrap_or(None), 
            file: row.get(4)?,
            device: row.get(5).unwrap_or(None),
            timestamp: row.get(6)?,
            pinned: row.get(7).unwrap_or(false),
        })
    }).map_err(|e| e.to_string())?;

    let mut history = Vec::new();
    for row in rows {
        history.push(row.map_err(|e| e.to_string())?);
    }
    Ok(history)
}

#[tauri::command]
fn delete_history_item(id: i64) -> Result<(), String> {
    let config = Config::new().map_err(|e| e.to_string())?;
    let conn = Connection::open(&config.history.db_path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM history WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_history() -> Result<(), String> {
    let config = Config::new().map_err(|e| e.to_string())?;
    let conn = Connection::open(&config.history.db_path).map_err(|e| e.to_string())?;
    // Option: Keep pinned items? For now delete everything as requested "Clear All"
    // If user wants to just clear unpinned, we can change logic.
    // Let's protect pinned items by default as that is standard behavior for "Pin"
    conn.execute("DELETE FROM history WHERE pinned = 0", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn toggle_pin(id: i64) -> Result<(), String> {
    let config = Config::new().map_err(|e| e.to_string())?;
    let conn = Connection::open(&config.history.db_path).map_err(|e| e.to_string())?;
    conn.execute("UPDATE history SET pinned = NOT pinned WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
struct AppInfo {
    name: String,
    version: String,
    identifier: String,
    tauri_version: String,
    github_url: String,
    license: String,
}

#[derive(Serialize)]
struct DependencyInfo {
    name: String,
    version: String,
    category: String,
}

/// 获取应用基本信息
#[tauri::command]
fn get_app_info() -> AppInfo {
    AppInfo {
        name: "SyncClipboard".to_string(),  // 移除"Rust版"，由前端翻译处理
        version: env!("CARGO_PKG_VERSION").to_string(),
        identifier: "com.syncclipboard.rs".to_string(),
        tauri_version: "2.9.5".to_string(),
        github_url: "https://github.com/SyncClipboard/sync_clipboard_rs".to_string(),
        license: "MIT".to_string(),
    }
}

/// 获取依赖库版本信息
#[tauri::command]
fn get_dependencies() -> Vec<DependencyInfo> {
    vec![
        DependencyInfo {
            name: "Tauri".to_string(),
            version: "2.9.5".to_string(),
            category: "UI Framework".to_string(),  // 英文
        },
        DependencyInfo {
            name: "Tokio".to_string(),
            version: "1.43.0".to_string(),
            category: "Async Runtime".to_string(),  // 英文
        },
        DependencyInfo {
            name: "Axum".to_string(),
            version: "0.8.8".to_string(),
            category: "HTTP Server".to_string(),  // 英文
        },
        DependencyInfo {
            name: "rusqlite".to_string(),
            version: "0.38.0".to_string(),
            category: "Database".to_string(),  // 英文
        },
        DependencyInfo {
            name: "mdns-sd".to_string(),
            version: "0.17.2".to_string(),
            category: "Service Discovery".to_string(),  // 英文
        },
        DependencyInfo {
            name: "reqwest".to_string(),
            version: "0.13.1".to_string(),
            category: "HTTP Client".to_string(),  // 英文
        },
    ]
}

/// 检查更新
#[tauri::command]
async fn check_update() -> Result<Option<String>, String> {
    let client = reqwest::Client::new();
    let url = "https://api.github.com/repos/SyncClipboard/sync_clipboard_rs/releases/latest";
    
    match client.get(url)
        .header("User-Agent", "SyncClipboard")
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(tag) = json.get("tag_name").and_then(|v| v.as_str()) {
                        let current = env!("CARGO_PKG_VERSION");
                        if tag.trim_start_matches('v') != current {
                            return Ok(Some(tag.to_string()));
                        }
                    }
                }
            }
            Ok(None)
        }
        Err(e) => Err(format!("检查更新失败: {}", e)),
    }
}


#[derive(Serialize)]
struct NetworkInterfaceInfo {
    name: String,
    ip: String,
    is_physical: bool,
}

#[derive(Serialize)]
struct NetworkInfo {
    interfaces: Vec<NetworkInterfaceInfo>,
    hostname: String,
}

#[tauri::command]
fn get_network_info() -> Result<NetworkInfo, String> {
    use local_ip_address::list_afinet_netifas;
    
    let mut interfaces = Vec::new();
    
    // 获取所有本地网卡信息
    if let Ok(ifaces) = list_afinet_netifas() {
        for (name, ip) in ifaces {
            let ip_str = ip.to_string();
            
            // 过滤掉环回地址
            if ip_str.starts_with("127.") || ip_str == "::1" {
                continue;
            }
            
            // 简单判断是否为物理网卡（根据名称常用前缀）
            // 例如 wlp (wifi), eth/enp/eno (ethernet)
            let is_physical = name.starts_with("wl") || 
                             name.starts_with("en") || 
                             name.starts_with("eth") ||
                             name.starts_with("wlan");
            
            interfaces.push(NetworkInterfaceInfo {
                name,
                ip: ip_str,
                is_physical,
            });
        }
    }
    
    // 排序：物理网卡优先
    interfaces.sort_by(|a, b| b.is_physical.cmp(&a.is_physical));
    
    let hostname = hostname::get()
        .ok()
        .and_then(|s| s.into_string().ok())
        .unwrap_or_else(|| "Unknown".to_string());
    
    Ok(NetworkInfo { interfaces, hostname })
}

#[derive(Serialize)]
struct LanDevice {
    name: String,
    ip: String,
    port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_active: Option<u64>, // Unix时间戳（秒）
}

#[tauri::command]
async fn get_lan_devices() -> Result<Vec<LanDevice>, String> {
    use clipboard_core::discovery::{DiscoveryService, Device};
    use clipboard_core::config::Config;
    
    // 获取配置
    let config = Config::new().map_err(|e| e.to_string())?;
    
    // 生成临时实例 ID
    let instance_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // 创建临时设备信息（用于发现）
    let temp_device = Device {
        id: config.general.device_id.clone(),
        name: format!("{}-Scanner", config.general.device_name),
        ip: "0.0.0.0".to_string(),
        port: 0,  // 扫描器不需要监听端口
        instance_id,
        capabilities: vec![],
    };
    
    // 创建发现服务
    let discovery = std::sync::Arc::new(
        DiscoveryService::new(temp_device)
            .await
            .map_err(|e| format!("Failed to create discovery service: {}", e))?
    );
    
    // 启动发现任务（在后台）
    let discovery_clone = discovery.clone();
    tokio::spawn(async move {
        discovery_clone.start().await;
    });
    
    // 等待发现任务启动
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // 主动发送扫描公告
    discovery.scan().await.map_err(|e| e.to_string())?;
    
    // 等待 3 秒收集设备响应
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // 获取发现的设备
    let devices = discovery.get_devices().await;
    
    // 转换为 LanDevice 格式
    Ok(devices.into_iter().map(|d| LanDevice {
        name: d.name,
        ip: d.ip,
        port: d.port,
        last_active: None, // 通过发现扫描的设备没有时间戳
    }).collect())
}

/// Get connected clients from the local server
/// This returns devices that have recently connected to this machine's server
#[tauri::command]
async fn get_connected_clients() -> Result<Vec<LanDevice>, String> {
    use clipboard_core::config::Config;
    
    // 获取配置以确定本地服务器地址
    let config = Config::new().map_err(|e| e.to_string())?;
    let server_url = format!("http://127.0.0.1:{}/api/connected_devices", config.server.port);
    
    // 请求已连接设备列表
    let client = reqwest::Client::new();
    let response = client
        .get(&server_url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch connected clients: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Server returned error: {}", response.status()));
    }
    
    // 解析响应
    #[derive(serde::Deserialize)]
    struct ConnectedClient {
        ip: String,
        device_name: Option<String>,
        user_agent: Option<String>,
        last_seen_timestamp: u64,
    }
    
    let clients: Vec<ConnectedClient> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    // 转换为 LanDevice 格式
    Ok(clients.into_iter().map(|c| LanDevice {
        name: c.device_name.unwrap_or_else(|| c.user_agent.unwrap_or_else(|| "Unknown Device".to_string())),
        ip: c.ip,
        port: 0, // 客户端没有监听端口
        last_active: Some(c.last_seen_timestamp),
    }).collect())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // .plugin(tauri_plugin_shell::init()) // Removed unused plugin
        .invoke_handler(tauri::generate_handler![
            get_config, 
            save_config,
            check_port_available,
            find_available_port,
            get_history, 
            get_network_info, 
            get_lan_devices,
            get_connected_clients,
            delete_history_item,
            clear_history,
            toggle_pin,
            get_app_info,
            get_dependencies,
            check_update
        ])
        .setup(|app| {
            // 1. Initialize Logging
            let subscriber = tracing_subscriber::fmt()
                .with_ansi(false)
                .finish();
            let _ = tracing::subscriber::set_global_default(subscriber);

            // 2. Setup System Tray
            use tauri::menu::{MenuBuilder, MenuItem};
            use tauri::tray::TrayIconBuilder;
            
            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;
            
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // 3. Handle window close event - minimize to tray instead of exit
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

             /* tauri::async_runtime::spawn(async move {
                // Cleanup Task (Disabled for now)
                let log_retention_days = Config::new().unwrap_or_else(|_| Config::new().unwrap()).history.log_retention_days;
                if log_retention_days > 0 {
                    let logs_path = log_dir.clone();
                    tokio::spawn(async move {
                        loop {
                             if let Ok(entries) = std::fs::read_dir(&logs_path) {
                                  let now = std::time::SystemTime::now();
                                  for entry in entries.flatten() {
                                       if let Ok(meta) = entry.metadata() {
                                            if let Ok(modified) = meta.modified() {
                                                 if let Ok(duration) = now.duration_since(modified) {
                                                      if duration.as_secs() > log_retention_days * 86400 {
                                                           let _ = std::fs::remove_file(entry.path());
                                                           println!("Cleaned up old log: {:?}", entry.path());
                                                      }
                                                 }
                                            }
                                       }
                                  }
                             }
                             // Run cleanup once a day (or on launch)
                             tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await; 
                        }
                    });
                }
            }); */

            // Start server in background
            tauri::async_runtime::spawn(async move {
                let config = Config::new().unwrap_or_else(|_| 
                    Config::new().expect("Failed to load config")
                ); 
                
                if config.server.enabled {
                    if let Err(e) = server::run(config.clone()).await {
                        tracing::error!("Background server error: {}", e);
                    }
                } else {
                    tracing::info!("Background server is disabled via config.");
                }
            });

            // Start Sync Manager (Client)
            tauri::async_runtime::spawn(async move {
                // Give server a moment to start
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let config = Config::new().unwrap_or_else(|_| 
                    Config::new().expect("Failed to load config")
                );
                
                if config.client.enabled {
                    match clipboard_core::clipboard_handler::ClipboardHandler::new() {
                        Ok(handler) => {
                            let handler = std::sync::Arc::new(handler);
                            let sync_manager = clipboard_core::sync::SyncManager::new(&config, handler);
                            tracing::info!("Starting Sync Manager (Client mode)...");
                            sync_manager.run().await;
                        },
                        Err(e) => tracing::error!("Failed to initialize clipboard handler: {}", e),
                    }
                } else {
                    tracing::info!("Sync Manager (Client) is disabled via config.");
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
