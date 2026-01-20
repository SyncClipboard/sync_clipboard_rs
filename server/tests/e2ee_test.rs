use clipboard_core::config::{Config, ServerConfig, AuthConfig, HistoryConfig, ClientConfig, GeneralConfig};
use clipboard_core::clipboard::ClipboardData;
use clipboard_core::crypto;
use base64::{Engine as _, engine::general_purpose};
use std::time::Duration;
use tokio::time::sleep;

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
            encrypt_password: None, // Server doesn't know password
        },
        history: HistoryConfig {
            max_count: 100,
            log_retention_days: 7,
            db_path: "test_e2ee.db".to_string(),
        },
        general: GeneralConfig {
            device_name: "TestE2EE".to_string(),
            device_id: "test-e2ee".to_string(),
        }
    };
    
    tokio::spawn(async move {
        if let Err(e) = server::run(config).await {
             eprintln!("Server error: {}", e);
        }
    });
    sleep(Duration::from_millis(500)).await;
}

#[tokio::test]
async fn test_e2ee_flow() {
    let port = 5098;
    start_test_server(port).await;
    
    let password = "mysecretpassword";
    let plain_text = "Hello E2EE World!";
    
    // 1. Client A: Manually encrypt and upload (simulating SyncManager)
    let encrypted_bytes = crypto::encrypt(plain_text.as_bytes(), password).unwrap();
    let b64 = general_purpose::STANDARD.encode(encrypted_bytes);
    let content_to_send = format!("E2EE::{}", b64);
    
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/SyncClipboard.json", port);
    
    let data = ClipboardData::new_text(content_to_send.clone());
    let resp = client.put(&url).json(&data).send().await.unwrap();
    assert!(resp.status().is_success());
    
    // 2. Server verification: Store should contain ENCRYPTED data
    let resp = client.get(&url).send().await.unwrap();
    let fetched_data: ClipboardData = resp.json().await.unwrap();
    
    if let ClipboardData::Text { content, .. } = fetched_data {
        assert!(content.starts_with("E2EE::"));
        assert_ne!(content, plain_text); // Must NOT be plain text
        assert_eq!(content, content_to_send);
        
        // 3. Client B: Manually decrypt (simulating SyncManager download)
        let b64_received = &content[6..];
        let bytes_received = general_purpose::STANDARD.decode(b64_received).unwrap();
        let decrypted_bytes = crypto::decrypt(&bytes_received, password).unwrap();
        let decrypted_text = String::from_utf8(decrypted_bytes).unwrap();
        
        assert_eq!(decrypted_text, plain_text);
        
        // 4. Decrypt with WRONG password should fail
        let wrong = crypto::decrypt(&bytes_received, "wrongpassword");
        assert!(wrong.is_err()); // HMAC verification should fail (implicitly via GCM auth tag)
    } else {
        panic!("Wrong data type");
    }
}
