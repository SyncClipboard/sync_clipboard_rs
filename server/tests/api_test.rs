use clipboard_core::config::{Config, ServerConfig, AuthConfig, ClientConfig, GeneralConfig};
use clipboard_core::clipboard::ClipboardData;
use std::time::Duration;
use tokio::time::sleep;

// Helper to start server in background
async fn start_test_server(port: u16, token: Option<String>) {
    let config = Config {
        server: ServerConfig {
            port,
            host: "127.0.0.1".to_string(),
            wevdav_enabled: false,
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
        history: clipboard_core::config::HistoryConfig {
            max_count: 100,
            log_retention_days: 7,
            db_path: format!("test_api_{}.db", port),
        },
        general: GeneralConfig {
            device_name: "TestAPI".to_string(),
            device_id: "test-api".to_string(),
        }
    };
    
    // Spawn server in background
    tokio::spawn(async move {
        if let Err(e) = server::run(config).await {
            eprintln!("Server error: {}", e);
        }
    });
    
    // Give it a moment to start
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_auth_protection() {
    let port = 5091;
    let token = "test_secret_token";
    start_test_server(port, Some(token.to_string())).await;
    
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/SyncClipboard.json", port);
    
    // 1. No token -> 401
    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    
    // 2. Wrong token -> 401
    let resp = client.get(&url)
        .header("Authorization", "Bearer wrong_token")
        .send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    
    // 3. Correct token -> 200 (or 404 if empty, checking status code is enough to pass auth)
    // Initially empty DB returns 404 Not Found, but Auth should pass.
    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send().await.unwrap();
        
    // 404 is allowed (Not Found), 200 is allowed. 401 is NOT allowed.
    assert_ne!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_long_polling() {
    let port = 5092;
    // No auth for simplicity in this test
    start_test_server(port, None).await;
    
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/SyncClipboard.json", port);
    
    // Initial set
    let data = ClipboardData::new_text("initial".to_string());
    client.put(&url).json(&data).send().await.unwrap();
    
    // Get initial ID
    let resp = client.get(&url).send().await.unwrap();
    let initial_id = resp.headers().get("X-Clipboard-Id").unwrap().to_str().unwrap().parse::<i64>().unwrap();
    
    // Spawn a long polling waiter
    let client_clone = client.clone();
    let url_clone = url.clone();
    let waiter = tokio::spawn(async move {
        let start = std::time::Instant::now();
        let resp = client_clone.get(&format!("{}?wait=5&last_id={}", url_clone, initial_id))
            .send().await.unwrap();
        let duration = start.elapsed();
        (resp, duration)
    });
    
    // Wait a bit to ensure waiter is waiting
    sleep(Duration::from_millis(500)).await;
    
    // Update clipboard
    let new_data = ClipboardData::new_text("updated".to_string());
    client.put(&url).json(&new_data).send().await.unwrap();
    
    // Waiter should resolve immediately
    let (resp, duration) = waiter.await.unwrap();
    assert!(resp.status().is_success());
    // Should be faster than the 5s timeout
    assert!(duration.as_secs() < 4);
    
    let body: ClipboardData = resp.json().await.unwrap();
    if let ClipboardData::Text { content, .. } = body {
        assert_eq!(content, "updated");
    } else {
        panic!("Wrong data type");
    }
}

#[tokio::test]
async fn test_file_upload_and_deduplication() {
    let port = 5093;
    let token = "token_files";
    start_test_server(port, Some(token.to_string())).await;
    
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);
    
    // 1. Upload a file
    let filename = "test_duplicate.png";
    let content = vec![1, 2, 3, 4, 5];
    
    let url = format!("{}/file/{}", base_url, filename);
    let resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .body(content.clone())
        .send().await.unwrap();
        
    assert!(resp.status().is_success());
    
    // 2. Upload same file again (Deduplication check)
    // We can't easily check if it skipped IO, but we verify it returns success
    let resp = client.put(&url)
        .header("Authorization", format!("Bearer {}", token))
        .body(content.clone())
        .send().await.unwrap();
    assert!(resp.status().is_success());
    
    // 3. Verify Download
    let resp = client.get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send().await.unwrap();
    assert!(resp.status().is_success());
    let downloaded_bytes = resp.bytes().await.unwrap();
    assert_eq!(downloaded_bytes, content); // Verify content matches
}
