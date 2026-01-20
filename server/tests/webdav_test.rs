//! WebDAV 功能测试
//! 测试 WebDAV 协议的基本操作（PROPFIND, PUT, GET, DELETE）

mod common;
use common::TestServer;

#[tokio::test]
async fn test_webdav_propfind() {
    let server = TestServer::with_webdav().await;
    let client = server.client();
    
    // PROPFIND 请求列出目录
    let url = format!("{}/webdav/", server.base_url);
    let resp = client.request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
        .header("Depth", "1")
        .send().await.unwrap();
    
    // WebDAV 应该返回 207 Multi-Status
    assert!(
        resp.status().is_success() || resp.status().as_u16() == 207,
        "PROPFIND should return 207 or success, got: {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_webdav_put_and_get() {
    let server = TestServer::with_webdav().await;
    let client = server.client();
    
    // PUT 上传文件
    let filename = "webdav_test.txt";
    let content = b"Hello WebDAV!";
    let url = format!("{}/webdav/{}", server.base_url, filename);
    
    let resp = client.put(&url)
        .body(content.to_vec())
        .send().await.unwrap();
    assert!(resp.status().is_success(), "PUT should succeed");
    
    // GET 下载文件
    let resp = client.get(&url).send().await.unwrap();
    assert!(resp.status().is_success(), "GET should succeed");
    
    let downloaded = resp.bytes().await.unwrap();
    assert_eq!(downloaded.as_ref(), content);
}

#[tokio::test]
async fn test_webdav_delete() {
    let server = TestServer::with_webdav().await;
    let client = server.client();
    
    // 先上传一个文件
    let filename = "to_delete.txt";
    let url = format!("{}/webdav/{}", server.base_url, filename);
    
    client.put(&url)
        .body(b"temporary".to_vec())
        .send().await.unwrap();
    
    // DELETE 删除文件
    let resp = client.delete(&url).send().await.unwrap();
    assert!(resp.status().is_success(), "DELETE should succeed");
    
    // 验证文件已删除
    let resp = client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::NOT_FOUND);
}
