use clipboard_core::config::{Config, ServerConfig, AuthConfig, HistoryConfig, ClientConfig, GeneralConfig};
use clipboard_core::clipboard::ClipboardData;
use std::time::Duration;
use tokio::time::sleep;

// Helper to start server in background
async fn start_test_server(port: u16) {
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
            token: None,
            encrypt_password: None,
        },
        history: HistoryConfig {
            max_count: 10,
            log_retention_days: 7,
            db_path: "test_rich.db".to_string(),
        },
        general: GeneralConfig {
            device_name: "TestRich".to_string(),
            device_id: "test-rich".to_string(),
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
async fn test_rich_text_sync() {
    let port = 5096;
    start_test_server(port).await;
    
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/SyncClipboard.json", port);
    
    // 1. Create content with HTML
    let text_content = "Bold Text".to_string();
    let html_content = "<b>Bold Text</b>".to_string();
    
    let data = ClipboardData::Text {
        content: text_content.clone(),
        html: Some(html_content.clone()),
        file: None,
        device: None,
    };
    
    // 2. Upload
    let resp = client.put(&url).json(&data).send().await.unwrap();
    assert!(resp.status().is_success());
    
    // 3. Download and Verify
    let resp = client.get(&url).send().await.unwrap();
    assert!(resp.status().is_success());
    
    let downloaded_data: ClipboardData = resp.json().await.unwrap();
    
    if let ClipboardData::Text { content, html, .. } = downloaded_data {
        assert_eq!(content, text_content);
        assert_eq!(html, Some(html_content));
        println!("Rich text (HTML) verified successfully!");
    } else {
        panic!("Wrong data type received");
    }
}
