import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Wifi, Monitor, Smartphone, RefreshCw, Copy, Check } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import { cn } from '../lib/utils';
import { useTranslation } from 'react-i18next';

interface InterfaceInfo {
    name: string;
    ip: string;
    is_physical: boolean;
}

interface NetworkInfo {
    interfaces: InterfaceInfo[];
    hostname: string;
}

interface LanDevice {
    name: string;
    ip: string;
    port: number;
    last_active?: number; // Unix timestamp
}

function Network() {
    const { t } = useTranslation();
    const [networkInfo, setNetworkInfo] = useState<NetworkInfo | null>(null);
    const [lanDevices, setLanDevices] = useState<LanDevice[]>([]);
    const [connectedClients, setConnectedClients] = useState<LanDevice[]>([]); // 新增：已连接设备
    const [loading, setLoading] = useState(false);
    const [copiedIp, setCopiedIp] = useState<string | null>(null);

    const fetchNetworkInfo = async () => {
        try {
            const info = await invoke<NetworkInfo>('get_network_info');
            setNetworkInfo(info);
        } catch (err) {
            console.error("Failed to fetch network info:", err);
        }
    };

    const scanLanDevices = async () => {
        setLoading(true);
        try {
            // 同时获取mDNS扫描设备和已连接设备
            const [mdnsDevices, connected] = await Promise.all([
                invoke<LanDevice[]>('get_lan_devices'),
                invoke<LanDevice[]>('get_connected_clients')
            ]);
            setLanDevices(mdnsDevices);
            setConnectedClients(connected);
        } catch (err) {
            console.error("Failed to scan devices:", err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchNetworkInfo();
        scanLanDevices();
    }, []);

    const handleCopyIp = (ip: string) => {
        navigator.clipboard.writeText(ip).then(() => {
            setCopiedIp(ip);
            setTimeout(() => setCopiedIp(null), 2000);
        });
    };

    return (
        <div className="space-y-6 pb-20 animate-in fade-in slide-in-from-bottom-4 duration-500">
            {/* Local Network Info */}
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                        <Monitor size={20} />
                        {t('settings.network.local_info', 'Local Network Information')}
                    </CardTitle>
                    <CardDescription>{t('settings.network.local_info_desc', 'Your device network details')}</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div>
                        <p className="text-sm font-medium text-muted-foreground mb-2">{t('settings.network.hostname', 'Hostname')}</p>
                        <p className="text-lg font-mono bg-secondary px-3 py-2 rounded border border-border">
                            {networkInfo?.hostname || t('settings.common.loading', 'Loading...')}
                        </p>
                    </div>
                    <div>
                        <p className="text-sm font-medium text-muted-foreground mb-2">{t('settings.network.local_ips', 'Local IP Addresses')}</p>
                        <div className="space-y-2">
                            {networkInfo?.interfaces.map((iface) => (
                                <div key={`${iface.name}-${iface.ip}`} className="flex items-center justify-between bg-secondary px-3 py-2 rounded border border-border group gap-4">
                                    <div className="flex flex-col">
                                        <div className="flex items-center gap-2">
                                            <span className="text-xs font-bold text-primary px-1.5 py-0.5 bg-primary/10 rounded uppercase tracking-wider">
                                                {iface.name}
                                            </span>
                                            {iface.is_physical && (
                                                <span className="text-[10px] bg-green-500/10 text-green-500 border border-green-500/20 px-1 rounded">
                                                    {t('settings.network.physical_adapter', 'Physical')}
                                                </span>
                                            )}
                                        </div>
                                        <span className="font-mono text-lg">{iface.ip}</span>
                                    </div>
                                    <Button
                                        size="sm"
                                        variant="ghost"
                                        onClick={() => handleCopyIp(iface.ip)}
                                        className={cn(
                                            "shrink-0 transition-all",
                                            copiedIp === iface.ip ? "bg-green-500 text-white" : "hover:bg-primary/20"
                                        )}
                                    >
                                        {copiedIp === iface.ip ? <Check size={16} /> : <Copy size={16} />}
                                    </Button>
                                </div>
                            ))}
                        </div>
                    </div>
                </CardContent>
            </Card>

            {/* LAN Devices Discovery */}
            <Card>
                <CardHeader>
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="flex items-center gap-2">
                                <Wifi size={20} />
                                {t('settings.network.lan_devices', 'LAN Devices')}
                            </CardTitle>
                            <CardDescription>{t('settings.network.lan_devices_desc', 'Discovered sync servers on your network')}</CardDescription>
                        </div>
                        <Button
                            onClick={scanLanDevices}
                            disabled={loading}
                            size="sm"
                            variant="outline"
                        >
                            <RefreshCw size={16} className={cn("mr-2", loading && "animate-spin")} />
                            {loading ? t('settings.network.scanning', 'Scanning...') : t('settings.network.rescan', 'Rescan')}
                        </Button>
                    </div>
                </CardHeader>
                <CardContent>
                    {lanDevices.length === 0 ? (
                        <div className="flex flex-col items-center justify-center py-12 text-muted-foreground opacity-60">
                            <Smartphone size={48} className="mb-4 opacity-50" />
                            <p className="text-sm">{t('settings.network.no_devices', 'No devices found on LAN')}</p>
                            <p className="text-xs mt-1">{t('settings.network.no_devices_hint', 'Make sure other devices have server enabled')}</p>
                        </div>
                    ) : (
                        <div className="space-y-3">
                            {lanDevices.map((device, idx) => (
                                <div
                                    key={idx}
                                    className="bg-secondary border border-border rounded-lg p-4 hover:border-primary/50 transition-colors"
                                >
                                    <div className="flex items-center justify-between">
                                        <div>
                                            <p className="font-medium">{device.name}</p>
                                            <p className="text-sm text-muted-foreground font-mono">
                                                {device.ip}:{device.port}
                                            </p>
                                        </div>
                                        <Button
                                            size="sm"
                                            variant="default"
                                            onClick={() => handleCopyIp(`${device.ip}:${device.port}`)}
                                        >
                                            {t('settings.network.copy_address', 'Copy Address')}
                                        </Button>
                                    </div>
                                </div>
                            ))}
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Connected Clients - 新增：已连接设备卡片 */}
            {connectedClients.length > 0 && (
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Smartphone size={20} />
                            {t('settings.network.connected_clients', '已连接设备')}
                        </CardTitle>
                        <CardDescription>{t('settings.network.connected_clients_desc', '最近连接到本机服务器的设备')}</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="space-y-3">
                            {connectedClients.map((device, idx) => (
                                <div
                                    key={idx}
                                    className="bg-secondary border border-border rounded-lg p-4 hover:border-primary/50 transition-colors"
                                >
                                    <div className="flex items-center justify-between">
                                        <div>
                                            <p className="font-medium">{device.name}</p>
                                            <p className="text-sm text-muted-foreground font-mono">
                                                {device.ip}
                                            </p>
                                            {device.last_active && (
                                                <p className="text-xs text-muted-foreground mt-1">
                                                    {t('settings.network.last_active', '最后活跃')}: {new Date(device.last_active * 1000).toLocaleString()}
                                                </p>
                                            )}
                                        </div>
                                        <Button
                                            size="sm"
                                            variant="ghost"
                                            onClick={() => handleCopyIp(device.ip)}
                                            className={cn(
                                                "shrink-0",
                                                copiedIp === device.ip ? "bg-green-500 text-white" : ""
                                            )}
                                        >
                                            {copiedIp === device.ip ? <Check size={16} /> : <Copy size={16} />}
                                        </Button>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </CardContent>
                </Card>
            )}
        </div>
    );
}

export default Network;
