use crate::clipboard_handler::ClipboardHandler;
use crate::clipboard::ClipboardData;
use crate::config::Config;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use reqwest::Client;
use anyhow::Result;
// use uuid::Uuid;
// use image::ImageEncoder;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose};
use crate::crypto;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub struct SyncManager {
    clipboard: Arc<ClipboardHandler>,
    client: Client,
    server_url: String,
    token: Option<String>,
    encrypt_password: Option<String>,
    device_name: String,
}

impl SyncManager {
    pub fn new(config: &Config, clipboard: Arc<ClipboardHandler>) -> Self {
        let server_url = format!("http://{}:{}/SyncClipboard.json", config.client.remote_host, config.client.remote_port);
        Self {
            clipboard,
            client: Client::new(),
            server_url,
            token: config.auth.token.clone(),
            encrypt_password: config.auth.encrypt_password.clone(),
            device_name: config.general.device_name.clone(),
        }
    }

    pub async fn run(&self) {
        let mut last_clipboard_content = String::new();
        let mut last_clipboard_html = String::new(); // Track HTML separately
        let mut last_image_hash = String::new(); // Track Image Hash
        let mut last_id: i64 = -1;

        // 0. Initial Sync (Prevent re-upload loop on restart)
        tracing::info!("Performing initial sync check...");
        match self.download(0, -1).await {
            Ok((Some(data), id)) => {
                last_id = id;
                match data {
                     ClipboardData::Text { content, html, .. } => {
                         last_clipboard_content = content.clone();
                         last_clipboard_html = html.unwrap_or_default();
                         tracing::info!("Initial sync: Loaded text from server (ID: {})", id);
                     },
                     ClipboardData::Image { hash, .. } => {
                         if let Some(h) = hash {
                             last_image_hash = h;
                             tracing::info!("Initial sync: Loaded image hash from server (ID: {})", id);
                         }
                     },
                     _ => {}
                }
            },
            Ok((None, id)) => {
                last_id = id;
                tracing::info!("Initial sync: Server empty or no change (ID: {})", id);
            },
            Err(e) => {
                tracing::warn!("Initial sync failed (offline?): {}", e);
            }
        }

        loop {
            // 1. Check Local Clipboard
            // We check text AND html. 
            let current_text = self.clipboard.get_text().unwrap_or_default();
            let current_html = self.clipboard.get_html().unwrap_or_default();
            
            // Only upload if something meaningful changed and is not empty (at least one of them)
            // But usually empty text means empty clipboard.
            let text_changed = current_text != last_clipboard_content && !current_text.is_empty();
            let html_changed = current_html != last_clipboard_html && !current_html.is_empty();

            if text_changed || html_changed {
                 tracing::info!("Local clipboard changed (Text: {}, HTML: {}). Uploading...", text_changed, html_changed);
                 
                 // Pass both text and html to upload
                 let html_opt = if current_html.is_empty() { None } else { Some(current_html.clone()) };
                 
                 if let Err(e) = self.upload_text(current_text.clone(), html_opt).await {
                    tracing::error!("Failed to upload text/html: {}", e);
                 } else {
                    last_clipboard_content = current_text;
                    last_clipboard_html = current_html;
                 }
            }

            // ... (keep image check)
             match self.clipboard.get_image() {
                Ok(image) => {
                    let _w = image.width();
                    let _h = image.height();
                    if let Ok(png_bytes) = self.encode_png(&image) {
                         let mut hasher = Sha256::new();
                         hasher.update(&png_bytes);
                         let hash = hex::encode(hasher.finalize());
                         
                         // Fix Loop: Only upload if hash is different from last one (downloaded or uploaded)
                         if hash != last_image_hash {
                             let filename = format!("{}.png", hash);
    
                             if let Err(e) = self.upload_image(filename, png_bytes, hash.clone()).await {
                                 tracing::error!("Failed to upload image: {}", e);
                             } else {
                                 tracing::info!("Uploaded image");
                                 last_image_hash = hash;
                             }
                         }
                    }
                }
                Err(_) => {}
            }

            // 2. Check Server (Long Polling)
            // Use wait=30s
            match self.download(30, last_id).await {
                 Ok((Some(data), new_id)) => {
                    last_id = new_id;
                    match data {
                        ClipboardData::Text { content, html, .. } => {
                            // Check if effectively different
                            let new_html = html.clone().unwrap_or_default();
                            if content != last_clipboard_content || new_html != last_clipboard_html {
                                tracing::info!("Server update (id={}). Updating local...", new_id);
                                
                                // If we have HTML, use set_html which sets both. Else set_text.
                                let result = if let Some(h) = html {
                                    self.clipboard.set_html(h, Some(content.clone()))
                                } else {
                                    self.clipboard.set_text(content.clone())
                                };
    
                                if let Err(e) = result {
                                    tracing::error!("Failed to set clipboard: {}", e);
                                } else {
                                    last_clipboard_content = content;
                                    last_clipboard_html = new_html;
                                }
                            }
                        },
                        ClipboardData::Image { hash, .. } => {
                             if let Some(h) = hash {
                                 // We don't necessarily download image content automatically to valid clipboard unless user wants it?
                                 // Currently logic didn't IMPLEMENT downloading image to clipboard in download() function above (it only handled Text).
                                 // Wait, the previous code block for download() ONLY handled ClipboardData::Text. 
                                 // If it was Image, it did nothing but update ID.
                                 // If we want to sync images, we must download the image content here!
                                 
                                 // For now, at least update the hash so we don't re-upload it if we ever implement download.
                                 // OR if the user manually copied it.
                                 last_image_hash = h;
                             }
                        },
                        _ => {}
                    }
                 }
                 Ok((None, id)) => {
                     // Update ID even if no data (e.g. timeout with known latest state?)
                     // Actually logic keeps last_id if None
                     if id > last_id { last_id = id; }
                 }
                 Err(e) => {
                     tracing::warn!("Failed to fetch from server: {}", e);
                     sleep(Duration::from_secs(5)).await; // Error backoff
                 },
            }
            
            // No sleep needed for long polling, but let's yield small time to avoid tight loop on errors or local checks
            sleep(Duration::from_millis(100)).await;
        }
    }

    pub async fn upload_text(&self, text: String, html: Option<String>) -> Result<()> {
        let content_to_send = if let Some(password) = &self.encrypt_password {
            // Encrypt
            match crypto::encrypt(text.as_bytes(), password) {
                Ok(encrypted_bytes) => {
                     let b64 = general_purpose::STANDARD.encode(encrypted_bytes);
                     format!("E2EE::{}", b64)
                },
                Err(e) => {
                    tracing::error!("Encryption failed: {}", e);
                    return Err(anyhow::anyhow!("Encryption failed: {}", e));
                }
            }
        } else {
            text
        };

        let mut data = ClipboardData::new_text(content_to_send);
        if let ClipboardData::Text { html: ref mut h, device: ref mut d, .. } = data {
            *d = Some(self.device_name.clone());
            
            // ... (E2EE logic)
            if let Some(password) = &self.encrypt_password {
                if let Some(raw_html) = html {
                     match crypto::encrypt(raw_html.as_bytes(), password) {
                        Ok(encrypted_bytes) => {
                             let b64 = general_purpose::STANDARD.encode(encrypted_bytes);
                             *h = Some(format!("E2EE::{}", b64));
                        },
                        Err(e) => {
                            tracing::error!("HTML Encryption failed: {}", e);
                            *h = None;
                        }
                    }
                }
            } else {
                *h = html;
            }
        }
        let mut req = self.client.put(&self.server_url);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        req.json(&data).send().await?;
        Ok(())
    }

    async fn upload_image(&self, filename: String, bytes: Vec<u8>, hash: String) -> Result<()> {
        // 1. Upload file
        let file_url = self.server_url.replace("SyncClipboard.json", &format!("file/{}", filename));
        let mut req_file = self.client.put(&file_url);
        if let Some(token) = &self.token {
            req_file = req_file.header("Authorization", format!("Bearer {}", token));
        }
        req_file.body(bytes).send().await?;

        // 2. Update metadata
        let data = ClipboardData::Image { 
            hash: Some(hash),
            filename: filename,
            device: Some(self.device_name.clone()),
        };
        let mut req_meta = self.client.put(&self.server_url);
        if let Some(token) = &self.token {
            req_meta = req_meta.header("Authorization", format!("Bearer {}", token));
        }
        req_meta.json(&data).send().await?;
        Ok(())
    }

    pub async fn upload_file_stream(&self, path: PathBuf, hash: String) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {:?}", path));
        }
        
        // 1. Prepare stream
        let file = File::open(&path).await?;
        let stream = ReaderStream::new(file);
        let body = reqwest::Body::wrap_stream(stream);

        // 2. Upload file
        let _filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown_file").to_string();
        // Use hash as filename on server for deduplication/storage (match upload_image logic?)
        // Actually upload_image uses hash.png. Here we use hash as filename maybe?
        // Or keep original extension.
        // Let's use hash + extension.
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
        let remote_filename = if extension.is_empty() { hash.clone() } else { format!("{}.{}", hash, extension) };

        let file_url = self.server_url.replace("SyncClipboard.json", &format!("file/{}", remote_filename));
        let mut req_file = self.client.put(&file_url);
        if let Some(token) = &self.token {
            req_file = req_file.header("Authorization", format!("Bearer {}", token));
        }
        
        // Note: Content-Length is usually required for some servers, but chunked transfer encoding (stream) works without it
        // unless server requires it. Axum handles streaming body fine.
        req_file.body(body).send().await?;

        // 3. Update metadata
        let data = ClipboardData::File { 
            hash: Some(hash),
            filename: remote_filename,
            device: Some(self.device_name.clone()),
        };
        let mut req_meta = self.client.put(&self.server_url);
        if let Some(token) = &self.token {
            req_meta = req_meta.header("Authorization", format!("Bearer {}", token));
        }
        req_meta.json(&data).send().await?;
        Ok(())
    }

    fn encode_png(&self, image: &image::DynamicImage) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;
        Ok(bytes)
    }

    pub async fn download(&self, wait: u64, last_id: i64) -> Result<(Option<ClipboardData>, i64)> {
        let url = format!("{}?wait={}&last_id={}", self.server_url, wait, last_id);
        let mut req = self.client.get(&url);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        if resp.status().is_success() {
            let id = resp.headers()
                .get("X-Clipboard-Id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
                
            let mut data: ClipboardData = resp.json().await?;
            
            // Decrypt if needed
            if let ClipboardData::Text { content, html, .. } = &mut data {
                 // 1. Decrypt Text
                 if content.starts_with("E2EE::") {
                     if let Some(password) = &self.encrypt_password {
                         let b64 = &content[6..];
                         if let Ok(bytes) = general_purpose::STANDARD.decode(b64) {
                             if let Ok(decrypted_bytes) = crypto::decrypt(&bytes, password) {
                                  if let Ok(decrypted_text) = String::from_utf8(decrypted_bytes) {
                                      *content = decrypted_text;
                                  }
                             }
                         }
                     }
                 }
                 
                 // 2. Decrypt HTML
                 if let Some(h) = html {
                     if h.starts_with("E2EE::") {
                         if let Some(password) = &self.encrypt_password {
                              let b64 = &h[6..];
                              if let Ok(bytes) = general_purpose::STANDARD.decode(b64) {
                                   if let Ok(decrypted_bytes) = crypto::decrypt(&bytes, password) {
                                       if let Ok(decrypted_html) = String::from_utf8(decrypted_bytes) {
                                           *html = Some(decrypted_html);
                                       }
                                   }
                              }
                         }
                     }
                 }
            }
            
            Ok((Some(data), id))
        } else if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
             Ok((None, last_id))
        } else {
             Ok((None, last_id))
        }
    }
}
