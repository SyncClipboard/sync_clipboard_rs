use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::db::Database;
use std::sync::Arc;
use clipboard_core::clipboard::ClipboardData;
use tokio::sync::Notify;

// Shared state
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub notify: Arc<Notify>,
    pub token: Option<String>,
    pub tracker: Arc<crate::client_tracker::ClientTracker>,
}

#[derive(Deserialize)]
pub struct PollQuery {
    pub wait: Option<u64>,
    pub last_id: Option<i64>,
}

#[derive(Serialize)]
pub struct HistoryItem {
    pub id: i64,
    pub r#type: String,
    pub content: Option<String>,
    pub file: Option<String>,
    pub hash: Option<String>,
    pub html: Option<String>,
    pub device: Option<String>,
    pub timestamp: String,
    pub pinned: bool,
}

pub async fn get_clipboard(
    State(state): State<AppState>,
    Query(query): Query<PollQuery>,
) -> Result<(axum::http::HeaderMap, Json<ClipboardData>), StatusCode> {
    let wait_time = query.wait.unwrap_or(0);
    let mut latest_id = state.db.get_latest_id().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.unwrap_or(0);
    
    // If long polling is requested
    if wait_time > 0 {
        let last_id = query.last_id.unwrap_or(-1);
        
        if latest_id == last_id {
            // Wait for notification or timeout
            let notify = state.notify.clone();
            let notified = notify.notified();
            
            tokio::select! {
                _ = notified => {
                    // Notified, fetch new ID
                     latest_id = state.db.get_latest_id().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.unwrap_or(0);
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)) => {
                    // Timeout
                }
            }
        }
    }

    match state.db.get_latest() {
        Ok(Some(data)) => {
            let mut headers = axum::http::HeaderMap::new();
            headers.insert("X-Clipboard-Id", latest_id.to_string().parse().unwrap());
            Ok((headers, Json(data)))
        },
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_clipboard(
    State(state): State<AppState>,
    Json(payload): Json<ClipboardData>,
) -> StatusCode {
    match state.db.save(&payload) {
        Ok(_) => {
            state.notify.notify_waiters();
            StatusCode::OK
        },
        Err(e) => {
            tracing::error!("Failed to save clipboard: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_history_list(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<HistoryItem>>, StatusCode> {
    let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let offset = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);

    let items = state.db.get_history(limit, offset).map_err(|e| {
        tracing::error!("Failed to fetch history: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let history: Vec<HistoryItem> = items.into_iter().map(|(id, type_, content, file, hash, html, device, pinned, timestamp)| {
        HistoryItem {
            id,
            r#type: type_,
            content,
            file,
            hash,
            html,
            device,
            pinned,
            timestamp,
        }
    }).collect();

    Ok(Json(history))
}

#[derive(Deserialize)]
pub struct PinUpdate {
    pub pinned: bool,
}

pub async fn delete_history(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> StatusCode {
    match state.db.delete_history(id) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to delete history: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn pin_history(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<PinUpdate>,
) -> StatusCode {
    match state.db.set_pinned(id, payload.pinned) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to pin history: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// ===== Discovery API for cross-subnet device discovery =====

/// Discovery information returned to help clients identify this server
#[derive(Serialize)]
pub struct DiscoveryInfo {
    /// Service identifier
    pub service: String,
    /// Server version
    pub version: String,
    /// Device name
    pub device_name: String,
    /// Device ID (placeholder for now)
    pub device_id: String,
    /// Server capabilities
    pub capabilities: Vec<String>,
}

/// Discovery endpoint - allows clients to identify this as a SyncClipboard server
/// This endpoint is designed for cross-subnet discovery via HTTP scanning
/// 
/// GET /api/discovery
/// Response: JSON with server information
pub async fn get_discovery_info(
    State(_state): State<AppState>,
) -> Json<DiscoveryInfo> {
    Json(DiscoveryInfo {
        service: "SyncClipboard".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        device_name: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "Unknown Device".to_string()),
        device_id: "auto-generated".to_string(), // TODO: get from config when available
        capabilities: vec!["clipboard".to_string(), "file".to_string(), "history".to_string()],
    })
}

/// Get connected devices endpoint
/// Returns a list of recently active clients (within 5 minutes)
/// 
/// GET /api/connected_devices
/// Response: JSON array of connected clients
pub async fn get_connected_devices(
    State(state): State<AppState>,
) -> Json<Vec<crate::client_tracker::ConnectedClient>> {
    let clients = state.tracker.get_active_clients().await;
    Json(clients)
}

