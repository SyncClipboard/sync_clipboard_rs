use clipboard_core::config::{Config, ServerConfig, AuthConfig, HistoryConfig, ClientConfig, GeneralConfig};
// use clipboard_core::clipboard::ClipboardData;
use std::time::Duration;
use tokio::time::sleep;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
// use std::path::PathBuf;
use tokio_util::io::ReaderStream;
use sha2::{Sha256, Digest};

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
            max_count: 100,
            log_retention_days: 7,
            db_path: "test_large.db".to_string(),
        },
        general: GeneralConfig {
            device_name: "TestLarge".to_string(),
            device_id: "test-large".to_string(),
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
async fn test_large_file_upload_stream() {
    let port = 5097;
    start_test_server(port).await;

    // 1. Create a "large" dummy file (e.g. 5MB for test speed, but mechanism is same)
    let file_size = 5 * 1024 * 1024;
    let filename = "large_test_file.bin";
    let path = std::env::temp_dir().join(filename);
    
    {
        let mut file = File::create(&path).await.unwrap();
        // Write random-ish data (chunks)
        let chunk = vec![0u8; 1024 * 1024]; // 1MB chunk
        for i in 0..5 {
             // Mutate chunk slightly to verify content matches
             let mut c = chunk.clone();
             c[0] = i as u8;
             file.write_all(&c).await.unwrap();
        }
        file.flush().await.unwrap();
    }
    
    // Calculate hash manually
    let mut hasher = Sha256::new();
    let content = tokio::fs::read(&path).await.unwrap();
    hasher.update(&content);
    let hash = hex::encode(hasher.finalize());
    let remote_filename = format!("{}.bin", hash); // assuming ext

    // 2. Simulate Client Upload (Streaming)
    // We strictly use streaming body to verify server accepts it
    let file = File::open(&path).await.unwrap();
    let stream = ReaderStream::new(file);
    let body = reqwest::Body::wrap_stream(stream);
    
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/file/{}", port, remote_filename);
    
    let resp = client.put(&url)
        .body(body)
        .send().await.unwrap();
        
    assert!(resp.status().is_success());
    
    // 3. Verify Server File Content
    let upload_dir = std::path::Path::new("uploads");
    let stored_path = upload_dir.join(&remote_filename);
    
    assert!(stored_path.exists());
    let stored_content = tokio::fs::read(&stored_path).await.unwrap();
    assert_eq!(stored_content.len(), file_size);
    assert_eq!(stored_content, content);
    
    // Cleanup
    let _ = tokio::fs::remove_file(path).await;
    let _ = tokio::fs::remove_file(stored_path).await;
    let _ = tokio::fs::remove_dir(upload_dir).await;
}
