import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { ExternalLink, RefreshCw, Github, Package } from 'lucide-react';
import { Card } from './ui/card';
import { Button } from './ui/button';

interface AppInfo {
    name: string;
    version: string;
    identifier: string;
    tauri_version: string;
    github_url: string;
    license: string;
}

interface DependencyInfo {
    name: string;
    version: string;
    category: string;
}

export default function About() {
    const { t } = useTranslation();
    const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
    const [dependencies, setDependencies] = useState<DependencyInfo[]>([]);
    const [updateVersion, setUpdateVersion] = useState<string | null>(null);
    const [checking, setChecking] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        loadAppInfo();
        loadDependencies();
    }, []);

    const loadAppInfo = async () => {
        try {
            const info = await invoke<AppInfo>('get_app_info');
            setAppInfo(info);
        } catch (err) {
            console.error('Failed to load app info:', err);
        }
    };

    const loadDependencies = async () => {
        try {
            const deps = await invoke<DependencyInfo[]>('get_dependencies');
            setDependencies(deps);
        } catch (err) {
            console.error('Failed to load dependencies:', err);
        }
    };

    const handleCheckUpdate = async () => {
        setChecking(true);
        setError(null);
        setUpdateVersion(null);

        try {
            const newVersion = await invoke<string | null>('check_update');
            if (newVersion) {
                setUpdateVersion(newVersion);
            } else {
                setError(t('about.upToDate', 'You\'re up to date!'));
            }
        } catch (err) {
            setError(t('about.checkUpdateFailed', 'Failed to check for updates, please try again later'));
            console.error('Check update error:', err);
        } finally {
            setChecking(false);
        }
    };

    if (!appInfo) {
        return (
            <div className="flex items-center justify-center h-64">
                <div className="text-muted-foreground">{t('common.loading', 'Loading...')}</div>
            </div>
        );
    }

    return (
        <div className="space-y-6">
            {/* App Info Card */}
            <Card className="p-6">
                <div className="flex items-start justify-between mb-6">
                    <div>
                        <h3 className="text-xl font-semibold mb-1">{appInfo.name}</h3>
                        <p className="text-sm text-muted-foreground">{t('about.subtitle', 'Cross-device clipboard sync tool')}</p>
                    </div>
                    <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center">
                        <Package className="w-6 h-6 text-primary" />
                    </div>
                </div>

                <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">{t('about.version', '版本')}</label>
                        <p className="text-sm font-mono">{appInfo.version}</p>
                    </div>
                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">{t('about.tauriVersion', 'Tauri 版本')}</label>
                        <p className="text-sm font-mono">{appInfo.tauri_version}</p>
                    </div>
                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">{t('about.identifier', '应用标识')}</label>
                        <p className="text-sm font-mono text-muted-foreground">{appInfo.identifier}</p>
                    </div>
                    <div className="space-y-1">
                        <label className="text-xs font-medium text-muted-foreground">{t('about.license', '许可证')}</label>
                        <p className="text-sm">{appInfo.license}</p>
                    </div>
                </div>
            </Card>

            {/* GitHub Link */}
            <Card className="p-6">
                <h3 className="font-semibold mb-4 flex items-center gap-2">
                    <Github className="w-4 h-4" />
                    {t('about.project', '项目地址')}
                </h3>
                <a
                    href={appInfo.github_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-2 text-sm text-primary hover:underline"
                >
                    {appInfo.github_url}
                    <ExternalLink className="w-3 h-3" />
                </a>
            </Card>

            {/* Update Check */}
            <Card className="p-6">
                <h3 className="font-semibold mb-4">{t('about.update', '更新')}</h3>
                <div className="space-y-4">
                    <Button
                        onClick={handleCheckUpdate}
                        disabled={checking}
                        variant="outline"
                        className="w-full sm:w-auto"
                    >
                        <RefreshCw className={`w-4 h-4 mr-2 ${checking ? 'animate-spin' : ''}`} />
                        {checking ? t('about.checking', '检查中...') : t('about.checkUpdate', '检查更新')}
                    </Button>

                    {updateVersion && (
                        <div className="p-4 bg-green-500/10 border border-green-500/20 rounded-lg">
                            <p className="text-sm font-medium text-green-600 dark:text-green-400 mb-2">
                                🎉 {t('about.newVersionAvailable', '发现新版本')}：{updateVersion}
                            </p>
                            <a
                                href={`${appInfo.github_url}/releases/latest`}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-sm text-primary hover:underline inline-flex items-center gap-1"
                            >
                                {t('about.download', '前往下载')}
                                <ExternalLink className="w-3 h-3" />
                            </a>
                        </div>
                    )}

                    {error && (
                        <div className="p-4 bg-muted rounded-lg">
                            <p className="text-sm text-muted-foreground">{error}</p>
                        </div>
                    )}
                </div>
            </Card>

            {/* Dependencies */}
            <Card className="p-6">
                <h3 className="font-semibold mb-4">{t('about.dependencies', '依赖库')}</h3>
                <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="border-b">
                                <th className="text-left py-2 px-4 font-medium text-muted-foreground">
                                    {t('about.name', '名称')}
                                </th>
                                <th className="text-left py-2 px-4 font-medium text-muted-foreground">
                                    {t('about.versionLabel', '版本')}
                                </th>
                                <th className="text-left py-2 px-4 font-medium text-muted-foreground">
                                    {t('about.category', '类别')}
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {dependencies.map((dep, index) => (
                                <tr key={index} className="border-b last:border-0 hover:bg-muted/50">
                                    <td className="py-3 px-4 font-medium">{dep.name}</td>
                                    <td className="py-3 px-4 font-mono text-muted-foreground">{dep.version}</td>
                                    <td className="py-3 px-4 text-muted-foreground">{dep.category}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </Card>
        </div>
    );
}
