use clipboard_core::config::*;
use tempfile::TempDir;
use std::net::TcpListener;

/// 测试服务器辅助结构
/// 自动管理端口分配、DB隔离和资源清理
pub struct TestServer {
    pub port: u16,
    pub base_url: String,
    _temp_dir: TempDir,
}

impl TestServer {
    /// 创建无认证的测试服务器
    pub async fn new() -> Self {
        Self::with_config(None, None, false).await
    }
    
    /// 创建带 Token 认证的测试服务器
    pub async fn with_token(token: impl Into<String>) -> Self {
        Self::with_config(Some(token.into()), None, false).await
    }

    /// 创建开启 WebDAV 的测试服务器
    pub async fn with_webdav() -> Self {
        Self::with_config(None, None, true).await
    }
    
    /// 创建自定义配置的测试服务器
    async fn with_config(token: Option<String>, max_count: Option<u32>, webdav_enabled: bool) -> Self {
        let port = Self::find_available_port();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path()
            .join(format!("test_{}.db", port))
            .to_string_lossy()
            .to_string();
        
        // Initialize logger if not already
        let _ = tracing_subscriber::fmt()
            .with_env_filter("server=debug,tower_http=debug")
            .try_init();

        let config = Config {
            server: ServerConfig {
                port,
                host: "127.0.0.1".to_string(),
                wevdav_enabled: webdav_enabled,
                tls: None,
                enabled: true,
            },
            client: ClientConfig {
                enabled: false,
                remote_host: "127.0.0.1".to_string(),
                remote_port: port,
            },
            auth: AuthConfig {
                username: None,
                password: None,
                token,
                encrypt_password: None,
            },
            history: HistoryConfig {
                max_count: max_count.unwrap_or(100),
                log_retention_days: 7,
                db_path,
            },
            general: GeneralConfig {
                device_name: "TestDevice".to_string(),
                device_id: format!("test-{}", port),
            },
        };
        

        
        // 启动服务器
        tokio::spawn(async move {
            if let Err(e) = server::run(config).await {
                eprintln!("Test server error: {}", e);
            }
        });
        
        // 等待服务器启动
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        Self {
            port,
            base_url: format!("http://127.0.0.1:{}", port),
            _temp_dir: temp_dir,
        }
    }
    
    /// 查找可用端口
    fn find_available_port() -> u16 {
        TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind to random port")
            .local_addr()
            .expect("Failed to get local addr")
            .port()
    }
    
    /// 创建 HTTP 客户端
    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
    
    /// 创建带认证头的 HTTP 客户端
    pub fn client_with_auth(&self, token: &str) -> reqwest::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );
        
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    }
}

// 测试结束时自动清理 (_temp_dir 会在 Drop 时删除)
