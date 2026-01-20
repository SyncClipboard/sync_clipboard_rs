use super::types::{DiscoveryPacket, AnnouncementPacket, Device, Result};
use std::sync::Arc;
use tokio::net::UdpSocket;

/// UDP Multicast 配置
pub const MULTICAST_ADDR: &str = "224.0.0.168";
pub const MULTICAST_PORT: u16 = 5354;
pub const ANNOUNCEMENT_INTERVAL_SECS: u64 = 30;

/// UDP Multicast 发现服务
pub struct MulticastService {
    /// UDP Socket
    socket: Arc<UdpSocket>,
    /// 本机设备信息
    my_device: Device,
}

impl MulticastService {
    /// 创建新的 Multicast 服务
    pub async fn new(my_device: Device) -> Result<Self> {
        use socket2::{Socket, Domain, Type, Protocol};
        
        tracing::info!("[Multicast] Initializing service for device: {} (port {})", my_device.name, my_device.port);
        
        // 创建原始 socket2 Socket
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        
        tracing::debug!("[Multicast] Setting SO_REUSEADDR and SO_REUSEPORT...");
        socket.set_reuse_address(true)?;
        #[cfg(not(target_os = "windows"))]
        socket.set_reuse_port(true)?;
        
        // 绑定到组播端口
        let bind_addr = format!("0.0.0.0:{}", MULTICAST_PORT);
        tracing::info!("[Multicast] Binding to {}...", bind_addr);
        let addr: std::net::SocketAddr = bind_addr.parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        socket.bind(&addr.into())?;
        
        // 设置为非阻塞（Tokio 需要）
        socket.set_nonblocking(true)?;
        
        // 转换为标准库 UdpSocket，然后再转为 TokioUdpSocket
        let std_socket: std::net::UdpSocket = socket.into();
        let socket = tokio::net::UdpSocket::from_std(std_socket)?;
        
        // 收集所有非环回 IPv4 地址并加入组播组
        tracing::info!("[Multicast] Joining multicast group on all interfaces...");
        use local_ip_address::list_afinet_netifas;
        
        if let Ok(ifaces) = list_afinet_netifas() {
            let multicast_addr: std::net::Ipv4Addr = MULTICAST_ADDR.parse()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
            let mut join_count = 0;
            
            for (name, ip) in ifaces {
                if let std::net::IpAddr::V4(v4_addr) = ip {
                    if !v4_addr.is_loopback() {
                        match socket.join_multicast_v4(multicast_addr, v4_addr) {
                            Ok(_) => {
                                tracing::info!("  ✓ Joined multicast on interface {} ({})", name, v4_addr);
                                join_count += 1;
                            }
                            Err(e) => {
                                tracing::warn!("  ✗ Failed to join multicast on {} ({}): {}", name, v4_addr, e);
                            }
                        }
                    }
                }
            }
            
            if join_count == 0 {
                tracing::error!("[Multicast] Failed to join multicast on ANY interface!");
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No valid network interfaces found for multicast").into());
            } else {
                tracing::info!("[Multicast] Successfully joined multicast on {} interface(s)", join_count);
            }
        } else {
            tracing::error!("[Multicast] Failed to enumerate network interfaces!");
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to list network interfaces").into());
        }
        
        tracing::info!("[Multicast] Service initialization complete");
        
        Ok(Self {
            socket: Arc::new(socket),
            my_device,
        })
    }

    /// 发送设备公告到所有网卡
    pub async fn send_announcement(&self) -> Result<()> {
        let packet = DiscoveryPacket::Announcement(AnnouncementPacket {
            device_id: self.my_device.id.clone(),
            alias: self.my_device.name.clone(),
            port: self.my_device.port,
            instance_id: self.my_device.instance_id,
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: self.my_device.capabilities.clone(),
        });
        
        self.broadcast_packet(packet).await
    }

    /// 发送搜索请求 (主动扫描) 到所有网卡
    pub async fn send_search(&self) -> Result<()> {
        let packet = DiscoveryPacket::Search(super::types::SearchPacket {
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        });
        
        self.broadcast_packet(packet).await
    }

    /// 辅助方法：通过所有网卡广播数据包
    async fn broadcast_packet(&self, packet: DiscoveryPacket) -> Result<()> {
        let json = serde_json::to_vec(&packet)?;
        let target = format!("{}:{}", MULTICAST_ADDR, MULTICAST_PORT);
        
        // 使用 spawn_blocking 在阻塞线程池中执行多网卡发送
        let target_clone = target.clone();
        let json_clone = json.clone();
        
        tokio::task::spawn_blocking(move || {
            use local_ip_address::list_afinet_netifas;
            use std::net::{UdpSocket, IpAddr};
            
            // 多路广播：在每个非环回接口上显式发送
            if let Ok(ifaces) = list_afinet_netifas() {
                for (name, ip) in ifaces {
                    if let IpAddr::V4(v4_addr) = ip {
                        if !v4_addr.is_loopback() {
                            // 为每个接口创建一个临时阻塞式 Socket 发送
                            match UdpSocket::bind((v4_addr, 0)) {
                                Ok(socket) => {
                                    socket.set_multicast_ttl_v4(32).ok();
                                    if let Err(e) = socket.send_to(&json_clone, &target_clone) {
                                        tracing::warn!("Failed to send discovery on interface {} ({}): {}", name, v4_addr, e);
                                    } else {
                                        tracing::debug!("Sent discovery on interface {} ({})", name, v4_addr);
                                    }
                                },
                                Err(e) => {
                                    tracing::warn!("Could not bind to interface {} ({}) for broadcast: {}", name, v4_addr, e);
                                }
                            }
                        }
                    }
                }
            }
        }).await.ok();
        
        Ok(())
    }

    /// 监听发现数据包
    pub async fn listen_discovery<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(Device) + Send + 'static,
    {
        let mut buf = vec![0u8; 2048];
        
        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, addr)) => {
                    // 解析 JSON 发现包
                    match serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) {
                        Ok(DiscoveryPacket::Announcement(packet)) => {
                            // 忽略自己的公告
                            if packet.device_id == self.my_device.id {
                                continue;
                            }
                            
                            let device = Device {
                                id: packet.device_id,
                                name: packet.alias,
                                ip: addr.ip().to_string(),
                                port: packet.port,
                                instance_id: packet.instance_id,
                                capabilities: packet.capabilities,
                            };
                            
                            callback(device);
                        }
                        Ok(DiscoveryPacket::Search(_)) => {
                            // 只有监听中的服务端才响应，或者非 Scanner 实例才响应
                            // 如果 port == 0 说明是扫描器，则不响应搜索，防止扫描器互换身份增加噪音
                            if self.my_device.port > 0 {
                                tracing::debug!("Received search request from {}, responding...", addr);
                                let _ = self.send_announcement().await;
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse discovery packet from {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("UDP recv error: {}", e);
                    return Err(e.into());
                }
            }
        }
    }

    /// 定期发送公告任务
    pub async fn periodic_announcement_task(self: Arc<Self>) {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(ANNOUNCEMENT_INTERVAL_SECS)
        );
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.send_announcement().await {
                tracing::error!("Failed to send periodic announcement: {}", e);
            }
        }
    }
}
