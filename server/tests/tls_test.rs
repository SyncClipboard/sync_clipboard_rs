use clipboard_core::config::{Config, ServerConfig, AuthConfig, TlsConfig, ClientConfig, GeneralConfig};
use std::time::Duration;
use tokio::time::sleep;
use std::process::Command;

async fn start_test_server(port: u16, cert_path: String, key_path: String) {
    let config = Config {
        server: ServerConfig {
            port,
            host: "127.0.0.1".to_string(),
            wevdav_enabled: false,
            tls: Some(TlsConfig {
                cert: cert_path,
                key: key_path,
            }),
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
        history: clipboard_core::config::HistoryConfig {
            max_count: 100,
            log_retention_days: 7,
            db_path: "test_tls.db".to_string(),
        },
        general: GeneralConfig {
            device_name: "TestTLS".to_string(),
            device_id: "test-tls".to_string(),
        }
    };
    
    // Server runs indefinitely, so we spawn it
    tokio::spawn(async move {
        if let Err(e) = server::run(config).await {
            // It might fail if port is taken, but for test we assume it works
            eprintln!("Server error: {}", e);
        }
    });
    sleep(Duration::from_millis(500)).await;
}

#[tokio::test]
async fn test_tls_connection() {
    // Generate certs
    let cert_path = "test_cert.pem";
    let key_path = "test_key.pem";
    
    // Cleanup first
    let _ = std::fs::remove_file(cert_path);
    let _ = std::fs::remove_file(key_path);

    // Use openssl to generate self-signed cert
    let status = Command::new("openssl")
        .args(&["req", "-x509", "-newkey", "rsa:2048", "-keyout", key_path, "-out", cert_path, "-days", "1", "-nodes", "-subj", "/CN=localhost"])
        .output()
        .expect("Failed to run openssl. Is it installed?");
        
    if !status.status.success() {
         panic!("OpenSSL failed: {}", String::from_utf8_lossy(&status.stderr));
    }

    let port = 5094;
    start_test_server(port, cert_path.to_string(), key_path.to_string()).await;
    
    // Client that accepts invalid certs (since it's self-signed)
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
        
    let url = format!("https://127.0.0.1:{}/SyncClipboard.json", port);
    
    // Attempt request
    let resp = client.get(&url).send().await;
    
    match resp {
        Ok(r) => {
            assert!(r.status().is_success() || r.status() == reqwest::StatusCode::NOT_FOUND);
            println!("HTTPS request successful!");
        },
        Err(e) => {
            panic!("HTTPS request failed: {}", e);
        }
    }
    
     // Cleanup
    let _ = std::fs::remove_file(cert_path);
    let _ = std::fs::remove_file(key_path);
}
