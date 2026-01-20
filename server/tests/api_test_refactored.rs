// 重构后的认证测试示例
// 使用新的 TestServer 工具，解决了 DB 共享和端口冲突问题

mod common;
use common::TestServer;
use clipboard_core::clipboard::ClipboardData;

#[tokio::test]
async fn test_auth_protection() {
    let token = "test_secret_token";
    let server = TestServer::with_token(token).await;
    
    let url = format!("{}/SyncClipboard.json", server.base_url);
    
    // 1. 无 Token -> 401
    let resp = server.client()
        .get(&url)
        .send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    
    // 2. 错误 Token -> 401
    let resp = server.client()
        .get(&url)
        .header("Authorization", "Bearer wrong_token")
        .send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
    
    // 3. 正确 Token -> 非401
    let resp = server.client()
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send().await.unwrap();
    assert_ne!(resp.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_long_polling() {
    let server = TestServer::new().await;
    let client = server.client();
    let url = format!("{}/SyncClipboard.json", server.base_url);
    
    // 初始化数据
    let data = ClipboardData::new_text("initial".to_string());
    client.put(&url).json(&data).send().await.unwrap();
    
    // 获取初始 ID
    let resp = client.get(&url).send().await.unwrap();
    let initial_id = resp.headers()
        .get("X-Clipboard-Id")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();
    
    // 启动长轮询等待器
    let client_clone = client.clone();
    let url_clone = url.clone();
    let waiter = tokio::spawn(async move {
        let start = std::time::Instant::now();
        let resp = client_clone
            .get(&format!("{}?wait=5&last_id={}", url_clone, initial_id))
            .send().await.unwrap();
        (resp, start.elapsed())
    });
    
    // 等待500ms确保等待器已启动
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // 更新数据
    let new_data = ClipboardData::new_text("updated".to_string());
    client.put(&url).json(&new_data).send().await.unwrap();
    
    // 验证等待器立即返回
    let (resp, duration) = waiter.await.unwrap();
    assert!(resp.status().is_success());
    assert!(duration.as_secs() < 4, "Long poll should return immediately");
    
    let body: ClipboardData = resp.json().await.unwrap();
    if let ClipboardData::Text { content, .. } = body {
        assert_eq!(content, "updated");
    } else {
        panic!("Wrong data type");
    }
}

#[tokio::test]
async fn test_file_upload_and_deduplication() {
    let token = "token_files";
    let server = TestServer::with_token(token).await;
    let client = server.client_with_auth(token);
    
    let filename = "test_duplicate.png";
    let content = vec![1, 2, 3, 4, 5];
    let url = format!("{}/file/{}", server.base_url, filename);
    
    // 1. 上传文件
    let resp = client.put(&url)
        .body(content.clone())
        .send().await.unwrap();
    assert!(resp.status().is_success());
    
    // 2. 再次上传相同文件（去重测试）
    let resp = client.put(&url)
        .body(content.clone())
        .send().await.unwrap();
    assert!(resp.status().is_success());
    
    // 3. 验证下载
    let resp = client.get(&url).send().await.unwrap();
    assert!(resp.status().is_success());
    let downloaded = resp.bytes().await.unwrap();
    assert_eq!(downloaded, content.as_slice());
}
