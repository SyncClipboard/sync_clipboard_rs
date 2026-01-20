//! 错误处理和边界条件测试

mod common;
use common::TestServer;
use clipboard_core::clipboard::ClipboardData;

#[tokio::test]
async fn test_invalid_json_payload() {
    let server = TestServer::new().await;
    let client = server.client();
    let url = format!("{}/SyncClipboard.json", server.base_url);
    
    // 发送无效的 JSON
    let resp = client.put(&url)
        .header("Content-Type", "application/json")
        .body("{invalid json}")
        .send().await.unwrap();
    
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_malformed_auth_header() {
    let token = "secret";
    let server = TestServer::with_token(token).await;
    let client = server.client();
    let url = format!("{}/SyncClipboard.json", server.base_url);
    
    // 错误的 Authorization 格式
    let resp = client.get(&url)
        .header("Authorization", "InvalidFormat")
        .send().await.unwrap();
    
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_empty_clipboard_data() {
    let server = TestServer::new().await;
    let client = server.client();
    let url = format!("{}/SyncClipboard.json", server.base_url);
    
    // 上传空内容
    let data = ClipboardData::new_text("".to_string());
    let resp = client.put(&url).json(&data).send().await.unwrap();
    assert!(resp.status().is_success());
    
    // 验证可以下载
    let resp = client.get(&url).send().await.unwrap();
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_unicode_filename() {
    let server = TestServer::new().await;
    let client = server.client();
    
    // 上传带 Unicode 字符的文件名
    let filename = "测试文件_🎉.txt";
    let url_encoded = format!("{}/file/{}", server.base_url, 
        urlencoding::encode(filename));
    
    let resp = client.put(&url_encoded)
        .body(b"Unicode test".to_vec())
        .send().await.unwrap();
    
    assert!(resp.status().is_success(), "Unicode filename should work");
}

#[tokio::test]
async fn test_zero_byte_file() {
    let server = TestServer::new().await;
    let client = server.client();
    
    let filename = "empty.bin";
    let url = format!("{}/file/{}", server.base_url, filename);
    
    // 上传空文件
    let resp = client.put(&url)
        .body(vec![])
        .send().await.unwrap();
    
    assert!(resp.status().is_success());
    
    // 验证下载
    let resp = client.get(&url).send().await.unwrap();
    assert!(resp.status().is_success());
    assert_eq!(resp.bytes().await.unwrap().len(), 0);
}

#[tokio::test]
async fn test_timeout_behavior() {
    let server = TestServer::new().await;
    let client = server.client();
    let url = format!("{}/SyncClipboard.json?wait=1&last_id=0", server.base_url);
    
    let start = std::time::Instant::now();
    let resp = client.get(&url).send().await.unwrap();
    let elapsed = start.elapsed();
    
    // 应该超时返回（~1秒）
    println!("Elapsed: {:?}", elapsed);
    assert!(elapsed.as_millis() >= 900 && elapsed.as_millis() <= 3500);
    // 超时应该返回 404 或 204
    assert!(
        resp.status() == reqwest::StatusCode::NOT_FOUND || 
        resp.status() == reqwest::StatusCode::NO_CONTENT
    );
}
