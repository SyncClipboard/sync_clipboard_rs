use super::types::{Device, DeviceEntry, DiscoveryMethod};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// 设备缓存管理器
pub struct DeviceCache {
    /// 设备缓存 (设备 ID -> 设备条目)
    devices: Arc<Mutex<HashMap<String, DeviceEntry>>>,
    /// 缓存生存时间（TTL）
    ttl: Duration,
}

impl DeviceCache {
    /// 创建新的设备缓存管理器
    pub fn new(ttl: Duration) -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    /// 注册或更新设备
    pub async fn register_device(&self, device: Device, method: DiscoveryMethod) {
        let mut devices = self.devices.lock().await;
        
        // 检查是否为重启设备
        if let Some(existing) = devices.get(&device.id) {
            if existing.device.instance_id != device.instance_id {
                tracing::info!(
                    "Device '{}' restarted (instance_id: {} -> {})",
                    device.name,
                    existing.device.instance_id,
                    device.instance_id
                );
            }
        } else {
            tracing::info!(
                "Discovered new device '{}' ({}) via {:?}",
                device.name,
                device.ip,
                method
            );
        }
        
        let entry = DeviceEntry {
            device,
            last_seen: Instant::now(),
            discovery_method: method,
        };
        
        devices.insert(entry.device.id.clone(), entry);
    }

    /// 获取所有在线设备
    pub async fn get_devices(&self) -> Vec<Device> {
        let devices = self.devices.lock().await;
        devices
            .values()
            .map(|entry| entry.device.clone())
            .collect()
    }

    /// 清理过期设备
    pub async fn cleanup_expired(&self) {
        let mut devices = self.devices.lock().await;
        let now = Instant::now();
        let ttl = self.ttl;
        
        let before_count = devices.len();
        devices.retain(|id, entry| {
            let expired = now.duration_since(entry.last_seen) >= ttl;
            if expired {
                tracing::info!(
                    "Device '{}' ({}) expired from cache",
                    entry.device.name,
                    id
                );
            }
            !expired
        });
        
        let removed = before_count - devices.len();
        if removed > 0 {
            tracing::debug!("Cleaned up {} expired device(s)", removed);
        }
    }

    /// 获取设备数量
    pub async fn count(&self) -> usize {
        self.devices.lock().await.len()
    }
}
