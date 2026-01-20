use super::cache::DeviceCache;
use super::multicast::{MulticastService, ANNOUNCEMENT_INTERVAL_SECS};
use super::types::{Device, DiscoveryMethod, Result};
use std::sync::Arc;
use std::time::Duration;

/// 设备缓存 TTL（90秒，3倍公告间隔）
const CACHE_TTL_SECS: u64 = ANNOUNCEMENT_INTERVAL_SECS * 3;

/// 缓存清理间隔（60秒）
const CLEANUP_INTERVAL_SECS: u64 = 60;

/// 混合发现服务
pub struct DiscoveryService {
    /// UDP Multicast 服务
    multicast: Arc<MulticastService>,
    /// 设备缓存
    cache: Arc<DeviceCache>,
    /// 本机设备信息（保留用于未来功能扩展）
    #[allow(dead_code)]
    my_device: Device,
}

impl DiscoveryService {
    /// 创建新的发现服务
    pub async fn new(device: Device) -> Result<Self> {
        // 创建 UDP Multicast 服务
        let multicast = Arc::new(MulticastService::new(device.clone()).await?);
        
        // 创建设备缓存（90秒 TTL）
        let cache = Arc::new(DeviceCache::new(Duration::from_secs(CACHE_TTL_SECS)));
        
        tracing::info!("Discovery service initialized for device '{}'", device.name);
        
        Ok(Self {
            multicast,
            cache,
            my_device: device,
        })
    }

    /// 启动所有发现任务
    pub async fn start(self: Arc<Self>) {
        tracing::info!("Starting discovery service tasks...");
        
        // 任务 1: UDP Multicast 定期公告 (仅当 port > 0 时)
        if self.my_device.port > 0 {
            let multicast_clone = self.multicast.clone();
            tokio::spawn(async move {
                tracing::info!("Starting periodic announcement task (every {}s)", ANNOUNCEMENT_INTERVAL_SECS);
                multicast_clone.periodic_announcement_task().await;
            });
        }
        
        // 任务 2: UDP Multicast 监听 (包括对搜索请求的响应)
        let multicast_clone = self.multicast.clone();
        let cache_clone = self.cache.clone();
        tokio::spawn(async move {
            tracing::info!("Starting UDP multicast discovery listener...");
            let _ = multicast_clone.listen_discovery(move |device| {
                let cache = cache_clone.clone();
                tokio::spawn(async move {
                    cache.register_device(device, DiscoveryMethod::UdpMulticast).await;
                });
            }).await;
        });
        
        // 任务 3: 定期清理过期缓存
        let cache_clone = self.cache.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));
            loop {
                interval.tick().await;
                cache_clone.cleanup_expired().await;
            }
        });
        
        // 立即发送一次公告 (仅当 port > 0 时)
        if self.my_device.port > 0 {
            if let Err(e) = self.multicast.send_announcement().await {
                tracing::error!("Failed to send initial announcement: {}", e);
            }
        }
        
        tracing::info!("All discovery tasks started successfully");
    }

    /// 获取已发现的设备列表
    pub async fn get_devices(&self) -> Vec<Device> {
        self.cache.get_devices().await
    }

    /// 手动触发扫描（发送搜索请求）
    pub async fn scan(&self) -> Result<()> {
        tracing::debug!("Manual scan triggered");
        self.multicast.send_search().await
    }

    /// 获取设备数量
    pub async fn device_count(&self) -> usize {
        self.cache.count().await
    }
}
