import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Config } from '../types';
import { Save, Loader2 } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Input } from "./ui/input"
import { Label } from "./ui/label"
import { Button } from "./ui/button"
import { Switch } from "./ui/switch"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./ui/card"

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select"
import { useTheme } from "./theme-provider"
import i18n from "../i18n";
import { cn } from "../lib/utils"

interface SettingsProps {
    activeSection: string;
}

function Settings({ activeSection }: SettingsProps) {
    const { t } = useTranslation();
    const { setTheme, theme } = useTheme();
    const [config, setConfig] = useState<Config>({
        server: { host: '0.0.0.0', port: 5033, tls: null, enabled: true },
        client: { enabled: false, remote_host: '127.0.0.1', remote_port: 5033 },
        auth: { token: '', encrypt_password: '' },
        history: { max_count: 100, log_retention_days: 7 },
        general: { device_name: 'Desktop' },
    });
    const [status, setStatus] = useState<{ msg: string, type: 'success' | 'error' | 'loading' | '' }>({ msg: '', type: '' });

    useEffect(() => {
        async function loadConfig() {
            try {
                const loaded = await invoke<Config>('get_config');
                setConfig(loaded);
            } catch (err) {
                console.error("Failed to load config:", err);
            }
        }
        loadConfig();
    }, []);

    const updateConfig = (section: keyof Config, field: string, value: any) => {
        setConfig(prev => ({
            ...prev,
            [section]: {
                ...prev[section],
                [field]: value
            }
        }));
    };

    const handleSave = async () => {
        try {
            setStatus({ msg: 'Saving...', type: 'loading' });
            await invoke('save_config', { config });
            setStatus({ msg: 'Saved!', type: 'success' });
            setTimeout(() => setStatus({ msg: '', type: '' }), 3000);
        } catch (err) {
            console.error("Failed to save:", err);
            setStatus({ msg: 'Error saving config', type: 'error' });
        }
    };

    return (
        <div className="space-y-6 pb-20 animate-in fade-in slide-in-from-bottom-4 duration-500">
            {activeSection === 'server' && (
                <>
                    <Card>
                        <CardHeader>
                            <CardTitle>{t('settings.server.title', 'Server Configuration')}</CardTitle>
                            <CardDescription>{t('settings.server.desc', 'Configure network bindings and ports')}</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex items-center space-x-2 border-b border-border pb-4 mb-4">
                                <Switch
                                    id="server-enabled"
                                    checked={config.server.enabled}
                                    onCheckedChange={(checked) => updateConfig('server', 'enabled', checked)}
                                />
                                <div className="flex flex-col">
                                    <Label htmlFor="server-enabled">{t('settings.server.enabled', 'Enable Sync Server')}</Label>
                                    <span className="text-[0.8rem] text-muted-foreground">{t('settings.server.enabled_desc', 'Allow other devices to connect to this device.')}</span>
                                </div>
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="host">{t('settings.server.host', 'Host Address')}</Label>
                                <Input
                                    id="host"
                                    value={config.server.host || ''}
                                    onChange={e => updateConfig('server', 'host', e.target.value)}
                                    placeholder="0.0.0.0"
                                />
                                <p className="text-[0.8rem] text-muted-foreground">{t('settings.server.host_desc', 'The IP address to bind to. 0.0.0.0 listens on all interfaces.')}</p>
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="port">{t('settings.server.port', 'Port')}</Label>
                                <Input
                                    id="port"
                                    type="number"
                                    value={config.server.port}
                                    onChange={e => updateConfig('server', 'port', parseInt(e.target.value))}
                                    placeholder="5033"
                                />
                            </div>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle>{t('settings.client.title', 'Client Configuration')}</CardTitle>
                            <CardDescription>{t('settings.client.desc', 'Connect to a remote sync server')}</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex items-center space-x-2 border-b border-border pb-4 mb-4">
                                <Switch
                                    id="client-enabled"
                                    checked={config.client.enabled}
                                    onCheckedChange={(checked) => updateConfig('client', 'enabled', checked)}
                                />
                                <div className="flex flex-col">
                                    <Label htmlFor="client-enabled">{t('settings.client.enabled', 'Enable Client Mode')}</Label>
                                    <span className="text-[0.8rem] text-muted-foreground">{t('settings.client.enabled_desc', 'Sync clipboard with a remote server.')}</span>
                                </div>
                            </div>
                            {config.client.enabled && (
                                <>
                                    <div className="grid gap-2">
                                        <Label htmlFor="remote-host">{t('settings.client.remote_host', 'Remote Server Address')}</Label>
                                        <Input
                                            id="remote-host"
                                            value={config.client.remote_host}
                                            onChange={e => updateConfig('client', 'remote_host', e.target.value)}
                                            placeholder="192.168.1.100"
                                        />
                                        <p className="text-[0.8rem] text-muted-foreground">{t('settings.client.remote_host_desc', 'IP address or hostname of the remote server.')}</p>
                                    </div>
                                    <div className="grid gap-2">
                                        <Label htmlFor="remote-port">{t('settings.client.remote_port', 'Remote Server Port')}</Label>
                                        <Input
                                            id="remote-port"
                                            type="number"
                                            value={config.client.remote_port}
                                            onChange={e => updateConfig('client', 'remote_port', parseInt(e.target.value))}
                                            placeholder="5033"
                                        />
                                    </div>
                                </>
                            )}
                        </CardContent>
                    </Card>
                </>
            )}

            {activeSection === 'security' && (
                <Card>
                    <CardHeader>
                        <CardTitle>{t('settings.security.title', 'Security & Encryption')}</CardTitle>
                        <CardDescription>{t('settings.security.desc', 'Protect your data with tokens and E2EE.')}</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-6">
                        <div className="grid gap-2">
                            <Label htmlFor="token">{t('settings.security.token', 'Authentication Token (Optional)')}</Label>
                            <Input
                                id="token"
                                type="password"
                                value={config.auth.token || ''}
                                onChange={e => updateConfig('auth', 'token', e.target.value)}
                                placeholder={t('settings.security.token_placeholder', 'Enter a secure token...')}
                            />
                        </div>

                        <div className="flex items-center space-x-2">
                            <Switch id="e2ee-mode" checked={!!config.auth.encrypt_password} onCheckedChange={(checked) => !checked && updateConfig('auth', 'encrypt_password', '')} />
                            <Label htmlFor="e2ee-mode">{t('settings.security.enable_e2ee', 'Enable End-to-End Encryption')}</Label>
                        </div>

                        <div className="grid gap-2">
                            <Label htmlFor="e2ee-pwd">{t('settings.security.e2ee_password', 'Encryption Password')}</Label>
                            <Input
                                id="e2ee-pwd"
                                type="password"
                                value={config.auth.encrypt_password || ''}
                                onChange={e => updateConfig('auth', 'encrypt_password', e.target.value)}
                                placeholder={t('settings.security.e2ee_placeholder', 'Must match on all devices...')}
                                disabled={false}
                            />
                            <p className="text-[0.8rem] text-muted-foreground">{t('settings.security.e2ee_desc', 'Your data is encrypted locally before being sent.')}</p>
                        </div>
                    </CardContent>
                </Card>
            )}

            {activeSection === 'general' && (
                <Card>
                    <CardHeader>
                        <CardTitle>{t('settings.general.title', 'Appearance & Language')}</CardTitle>
                        <CardDescription>{t('settings.general.desc', 'Customize the application look and feel.')}</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid gap-2">
                            <Label htmlFor="device-name">{t('settings.general.device_name', 'Device Name')}</Label>
                            <Input
                                id="device-name"
                                value={config.general.device_name || ''}
                                onChange={e => updateConfig('general', 'device_name', e.target.value)}
                                placeholder="My Device"
                            />
                            <p className="text-[0.8rem] text-muted-foreground">{t('settings.general.device_name_desc', 'Name displayed on other devices.')}</p>
                        </div>

                        <div className="grid gap-2">
                            <Label>{t('settings.common.language', 'Interface Language')}</Label>
                            <Select
                                value={i18n.language && i18n.language.startsWith('zh') ? 'zh' : 'en'}
                                onValueChange={(val) => {
                                    i18n.changeLanguage(val);
                                    // No reload needed, react-i18next handles re-render
                                }}
                            >
                                <SelectTrigger>
                                    <SelectValue placeholder={t('settings.common.select_language', 'Select Language')} />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="en">English (US)</SelectItem>
                                    <SelectItem value="zh">中文 (简体)</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>

                        <div className="grid gap-2">
                            <Label>{t('settings.common.theme', 'Theme Preference')}</Label>
                            <Select value={theme} onValueChange={(val) => setTheme(val as any)}>
                                <SelectTrigger>
                                    <SelectValue placeholder={t('settings.common.select_theme', 'Select Theme')} />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="light">{t('settings.common.theme_light', 'Light Mode')}</SelectItem>
                                    <SelectItem value="dark">{t('settings.common.theme_dark', 'Dark Mode')}</SelectItem>
                                    <SelectItem value="system">{t('settings.common.theme_system', 'System Default')}</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </CardContent>
                </Card>
            )}

            {activeSection === 'local' && (
                <Card>
                    <CardHeader>
                        <CardTitle>{t('settings.storage.title', 'Local Storage')}</CardTitle>
                        <CardDescription>{t('settings.storage.desc', 'Manage history retention.')}</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid gap-2">
                            <Label htmlFor="limit">{t('settings.storage.limit', 'History Limit')}</Label>
                            <Input
                                id="limit"
                                type="number"
                                value={config.history.max_count}
                                onChange={e => updateConfig('history', 'max_count', parseInt(e.target.value))}
                            />
                            <p className="text-[0.8rem] text-muted-foreground">{t('settings.storage.limit_desc', 'Maximum items to keep in history.')}</p>
                        </div>

                        <div className="grid gap-2">
                            <Label htmlFor="retention">{t('settings.storage.retention', 'Log Retention (Days)')}</Label>
                            <Input
                                id="retention"
                                type="number"
                                value={config.history.log_retention_days || 7}
                                onChange={e => updateConfig('history', 'log_retention_days', parseInt(e.target.value))}
                            />
                            <p className="text-[0.8rem] text-muted-foreground">{t('settings.storage.retention_desc', 'Days to keep log files before auto-deletion.')}</p>
                        </div>
                    </CardContent>
                </Card>
            )}

            {/* Action Bar */}
            <div className="fixed bottom-6 right-8 flex items-center gap-4 z-50">
                {status.msg && (
                    <div className={cn(
                        "px-4 py-2 rounded-md text-sm font-medium shadow-md animate-in fade-in slide-in-from-bottom-2",
                        status.type === 'success' ? "bg-green-500 text-white" : "bg-destructive text-white"
                    )}>
                        {status.msg === 'Saved!' ? t('settings.actions.saved', 'Saved!') :
                            status.msg === 'Saving...' ? t('settings.actions.saving', 'Saving...') :
                                status.msg === 'Error saving config' ? t('settings.actions.error', 'Error saving config') :
                                    status.msg}
                    </div>
                )}
                <Button onClick={handleSave} disabled={status.type === 'loading'} size="lg" className="shadow-xl">
                    {status.type === 'loading' ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Save className="mr-2 h-4 w-4" />}
                    {t('settings.actions.save', 'Save Changes')}
                </Button>
            </div>
        </div>
    );
}

export default Settings;
