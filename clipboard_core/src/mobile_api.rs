use crate::sync::SyncManager;
use crate::config::{Config, ServerConfig, AuthConfig, HistoryConfig};
use crate::clipboard_handler::ClipboardHandler;
use crate::clipboard::ClipboardData;
use std::sync::Arc;
use anyhow::anyhow;
use uniffi;
use thiserror::Error;

#[derive(uniffi::Record)]
pub struct MobileClipboardData {
    pub content: String,
    pub html: Option<String>,
    pub is_file: bool,
}

#[derive(uniffi::Object)]
pub struct MobileSyncManager {
    inner: SyncManager,
}

#[derive(Debug, Error, uniffi::Error)]
pub enum MobileError {
    #[error("General error: {0}")]
    General(String),
}

impl From<anyhow::Error> for MobileError {
    fn from(err: anyhow::Error) -> Self {
        MobileError::General(err.to_string())
    }
}

#[uniffi::export]
impl MobileSyncManager {
    #[uniffi::constructor]
    pub fn new(server_url: String, server_port: u16, token: String, encrypt_password: String) -> Result<Self, MobileError> {
        let config = Config {
            server: ServerConfig {
                host: server_url.clone(),
                port: server_port,
                wevdav_enabled: false,
                tls: None,
                enabled: true,
            },
            client: crate::config::ClientConfig {
                enabled: true,
                remote_host: server_url,
                remote_port: server_port,
            },
            auth: AuthConfig {
                username: None,
                password: None,
                token: if token.is_empty() { None } else { Some(token) },
                encrypt_password: if encrypt_password.is_empty() { None } else { Some(encrypt_password) },
            },
            history: HistoryConfig {
                max_count: 100,
                log_retention_days: 7,
                db_path: "mobile_history.db".to_string(),
            },
            general: crate::config::GeneralConfig {
                device_name: "Mobile".to_string(),
                device_id: uuid::Uuid::new_v4().to_string(),
            },
        };

        let clipboard = ClipboardHandler::new().map_err(|e| anyhow!("Failed to init clipboard: {}", e))?;
        let manager = SyncManager::new(&config, Arc::new(clipboard));
        
        Ok(Self { inner: manager })
    }

    /// Upload text to server
    pub async fn upload_text(&self, text: String, html: Option<String>) -> Result<(), MobileError> {
        self.inner.upload_text(text, html).await.map_err(MobileError::from)
    }

    /// Check for updates from server. Returns new content if any.
    /// Mobile app calls this periodically or on push notification.
    pub async fn check_updates(&self, wait: u64, last_id: i64) -> Result<Option<MobileClipboardData>, MobileError> {
        let (data, _) = self.inner.download(wait, last_id).await.map_err(MobileError::from)?;
        
        if let Some(d) = data {
            match d {
                ClipboardData::Text { content, html, .. } => {
                    Ok(Some(MobileClipboardData {
                        content,
                        html,
                        is_file: false,
                    }))
                },
                ClipboardData::Image { .. } => {
                    // TODO: Handle image download for mobile
                    Ok(None)
                },
                ClipboardData::File { .. } => {
                     Ok(Some(MobileClipboardData {
                        content: "File received (not supported in mobile lib yet)".to_string(),
                        html: None,
                        is_file: true,
                    }))
                }
            }
        } else {
            Ok(None)
        }
    }
}
