use serde::{Deserialize, Serialize};
use std::time::Instant;

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// 设备唯一标识
    pub id: String,
    /// 设备名称
    pub name: String,
    /// IP 地址
    pub ip: String,
    /// 服务端口
    pub port: u16,
    /// 实例 ID（用于重启检测）
    pub instance_id: u64,
    /// 能力列表
    pub capabilities: Vec<String>,
}

/// UDP 发现数据包封装
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DiscoveryPacket {
    /// 搜索请求（由客户端发起）
    #[serde(rename = "search")]
    Search(SearchPacket),
    /// 设备公告（由服务端发起）
    #[serde(rename = "announcement")]
    Announcement(AnnouncementPacket),
}

/// 搜索数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPacket {
    /// 要求的协议版本（可选）
    pub version: Option<String>,
}

/// UDP 公告数据包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementPacket {
    /// 设备唯一标识
    pub device_id: String,
    /// 设备别名/名称
    pub alias: String,
    /// 服务端口
    pub port: u16,
    /// 实例 ID（用于重启检测）
    pub instance_id: u64,
    /// 协议版本
    pub version: String,
    /// 能力列表
    pub capabilities: Vec<String>,
}

/// 设备缓存条目
#[derive(Debug, Clone)]
pub(crate) struct DeviceEntry {
    /// 设备信息
    pub device: Device,
    /// 最后发现时间
    pub last_seen: Instant,
    /// 发现方法（保留用于未来统计和调试）
    #[allow(dead_code)]
    pub discovery_method: DiscoveryMethod,
}

/// 设备发现方法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// mDNS 发现
    Mdns,
    /// UDP Multicast 发现
    UdpMulticast,
}

/// 发现服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("mDNS error: {0}")]
    Mdns(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, DiscoveryError>;
