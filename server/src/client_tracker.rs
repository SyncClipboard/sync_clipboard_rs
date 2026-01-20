use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tokio::sync::RwLock;
use serde::Serialize;

/// 已连接的客户端信息
#[derive(Clone, Serialize)]
pub struct ConnectedClient {
    pub ip: String,
    pub device_name: Option<String>,
    pub user_agent: Option<String>,
    #[serde(skip)]
    pub last_seen: SystemTime,
    /// 最后活跃时间（秒级时间戳，用于前端显示）
    pub last_seen_timestamp: u64,
}

/// 全局客户端追踪器
pub struct ClientTracker {
    clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,
}

impl ClientTracker {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录客户端访问
    pub async fn record_client(
        &self,
        ip: String,
        device_name: Option<String>,
        user_agent: Option<String>,
    ) {
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut clients = self.clients.write().await;
        
        // 如果已存在该IP的记录，且新设备名为None，则保留旧设备名
        let final_device_name = if let Some(existing) = clients.get(&ip) {
            device_name.or(existing.device_name.clone())
        } else {
            device_name
        };

        // 同理保留UserAgent（可选，通常UserAgent每次请求都会带）
        // 但为了保险起见也可以保留
        let final_user_agent = if let Some(existing) = clients.get(&ip) {
            user_agent.or(existing.user_agent.clone())
        } else {
            user_agent
        };

        let client = ConnectedClient {
            ip: ip.clone(),
            device_name: final_device_name,
            user_agent: final_user_agent,
            last_seen: now,
            last_seen_timestamp: timestamp,
        };

        clients.insert(ip, client);
    }

    /// 获取所有活跃客户端（5分钟内有活动）
    pub async fn get_active_clients(&self) -> Vec<ConnectedClient> {
        let mut clients = self.clients.write().await;
        let now = SystemTime::now();
        let timeout = Duration::from_secs(300); // 5分钟

        // 清理过期客户端
        clients.retain(|_, client| {
            now.duration_since(client.last_seen).unwrap_or(timeout) < timeout
        });

        // 返回活跃客户端列表
        clients.values().cloned().collect()
    }
}

impl Default for ClientTracker {
    fn default() -> Self {
        Self::new()
    }
}
