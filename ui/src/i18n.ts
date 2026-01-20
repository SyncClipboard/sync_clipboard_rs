import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
        resources: {
            en: {
                translation: {
                    "nav": {
                        "history": "History",
                        "network": "Network",
                        "configuration": "CONFIGURATION",
                        "general": "General",
                        "connection": "Connection",
                        "security": "Security",
                        "storage": "Storage"
                    },
                    "desc": {
                        "history": "View and manage your synced content",
                        "network": "View network information and discover devices",
                        "general": "Customize the application look and feel",
                        "connection": "Configure network options",
                        "security": "Manage encryption and access",
                        "storage": "Local database preferences"
                    },
                    "settings": {
                        "actions": {
                            "save": "Save Changes",
                            "saving": "Saving...",
                            "saved": "Saved!",
                            "error": "Error saving config"
                        },
                        "common": {
                            "language": "Interface Language",
                            "select_language": "Select Language",
                            "theme": "Theme Preference",
                            "select_theme": "Select Theme",
                            "theme_light": "Light Mode",
                            "theme_dark": "Dark Mode",
                            "theme_system": "System Default",
                            "loading": "Loading..."
                        },
                        "server": {
                            "title": "Server Configuration",
                            "desc": "Configure network bindings and ports",
                            "enabled": "Enable Sync Server",
                            "enabled_desc": "Allow other devices to connect to this device.",
                            "host": "Host Address",
                            "port": "Port",
                            "host_desc": "The IP address to bind to. 0.0.0.0 listens on all interfaces."
                        },
                        "client": {
                            "title": "Client Configuration",
                            "desc": "Connect to a remote sync server",
                            "enabled": "Enable Client Mode",
                            "enabled_desc": "Sync clipboard with a remote server.",
                            "remote_host": "Remote Server Address",
                            "remote_host_desc": "IP address or hostname of the remote server.",
                            "remote_port": "Remote Server Port"
                        },
                        "general": {
                            "title": "Appearance & Language",
                            "desc": "Customize the application look and feel",
                            "device_name": "Device Name",
                            "device_name_desc": "Name displayed on other devices."
                        },
                        "security": {
                            "token": "Authentication Token (Optional)",
                            "token_placeholder": "Enter a secure token...",
                            "enable_e2ee": "Enable End-to-End Encryption",
                            "e2ee_password": "Encryption Password",
                            "e2ee_placeholder": "Must match on all devices...",
                            "e2ee_desc": "Your data is encrypted locally before being sent."
                        },
                        "storage": {
                            "title": "Local Storage",
                            "desc": "Manage history retention.",
                            "limit": "History Limit",
                            "limit_desc": "Maximum items to keep in history.",
                            "retention": "Log Retention (Days)",
                            "retention_desc": "Days to keep log files before auto-deletion."
                        },
                        "network": {
                            "local_info": "Local Network Information",
                            "local_info_desc": "Your device network details",
                            "hostname": "Hostname",
                            "local_ips": "Local IP Addresses",
                            "lan_devices": "LAN Devices",
                            "lan_devices_desc": "Discovered sync servers on your network",
                            "scanning": "Scanning...",
                            "rescan": "Rescan",
                            "no_devices": "No devices found on LAN",
                            "no_devices_hint": "Make sure other devices have server enabled",
                            "copy_address": "Copy Address",
                            "physical_adapter": "Physical",
                            "connected_clients": "Connected Devices",
                            "connected_clients_desc": "Devices recently connected to local server",
                            "last_active": "Last Active"
                        },
                        "history": {
                            "recent_items": "Recent Items",
                            "clear_all": "Clear History",
                            "confirm_clear": "Are you sure you want to clear unpinned history?",
                            "pin": "Pin",
                            "unpin": "Unpin",
                            "delete": "Delete",
                            "empty": "No clipboard history found",
                            "empty_content": "Empty Content",
                            "clipboard_image": "Clipboard Image",
                            "open_full_size": "Open full size",
                            "unknown_file": "Unknown File",
                            "copy_to_clipboard": "Copy to Clipboard"
                        },
                        "about": {
                            "subtitle": "Cross-device clipboard sync tool",
                            "version": "Version",
                            "tauriVersion": "Tauri Version",
                            "identifier": "App Identifier",
                            "license": "License",
                            "project": "Project",
                            "update": "Update",
                            "checking": "Checking...",
                            "checkUpdate": "Check for Updates",
                            "upToDate": "You're up to date!",
                            "checkUpdateFailed": "Failed to check for updates",
                            "newVersionAvailable": "New version available",
                            "download": "Download",
                            "dependencies": "Dependencies",
                            "name": "Name",
                            "category": "Category"
                        }
                    }
                }
            },
            zh: {
                translation: {
                    "nav": {
                        "history": "历史记录",
                        "network": "网络设置",
                        "configuration": "配置选项",
                        "general": "通用设置",
                        "connection": "连接设置",
                        "security": "安全设置",
                        "storage": "存储设置"
                    },
                    "desc": {
                        "history": "查看并管理同步的剪贴板内容",
                        "network": "查看网络信息并发现设备",
                        "general": "自定义应用程序的外观与语言",
                        "connection": "配置服务器连接地址与端口",
                        "security": "管理端到端加密与访问令牌",
                        "storage": "本地数据库与历史记录保留策略"
                    },
                    "settings": {
                        "actions": {
                            "save": "保存更改",
                            "saving": "保存中...",
                            "saved": "已保存!",
                            "error": "保存配置失败",
                            "dependencies": "Dependencies",
                            "name": "Name",
                            "versionLabel": "Version",
                            "category": "Category",
                            // 依赖类别翻译（用于前端显示）
                            "category_ui_framework": "UI Framework",
                            "category_async_runtime": "Async Runtime",
                            "category_http_server": "HTTP Server",
                            "category_database": "Database",
                            "category_service_discovery": "Service Discovery",
                            "category_http_client": "HTTP Client"
                        },
                        "common": {
                            "language": "界面语言",
                            "select_language": "选择语言",
                            "theme": "主题偏好",
                            "select_theme": "选择主题",
                            "theme_light": "浅色模式",
                            "theme_dark": "深色模式",
                            "theme_system": "跟随系统",
                            "loading": "加载中..."
                        },
                        "server": {
                            "title": "服务器配置",
                            "desc": "配置网络监听地址与端口",
                            "enabled": "启用同步服务",
                            "enabled_desc": "允许其他设备连接到此设备。",
                            "host": "监听地址",
                            "port": "端口",
                            "host_desc": "绑定的IP地址。0.0.0.0 表示监听所有接口。"
                        },
                        "client": {
                            "title": "客户端配置",
                            "desc": "连接到远程同步服务器",
                            "enabled": "启用客户端模式",
                            "enabled_desc": "与远程服务器同步剪贴板。",
                            "remote_host": "远程服务器地址",
                            "remote_host_desc": "远程服务器的IP地址或主机名。",
                            "remote_port": "远程服务器端口"
                        },
                        "general": {
                            "title": "外观与语言",
                            "desc": "自定义应用程序的外观与体验",
                            "device_name": "设备名称",
                            "device_name_desc": "在其他设备上显示的名称。"
                        },
                        "security": {
                            "title": "安全与加密",
                            "desc": "使用令牌和端到端加密保护您的数据。",
                            "token": "认证令牌 (可选)",
                            "token_placeholder": "输入安全令牌...",
                            "enable_e2ee": "启用端到端加密",
                            "e2ee_password": "加密密码",
                            "e2ee_placeholder": "必须在所有设备上匹配...",
                            "e2ee_desc": "您的数据在发送前将在本地进行加密。"
                        },
                        "storage": {
                            "title": "本地存储",
                            "desc": "管理历史记录保留策略。",
                            "limit": "历史记录上限",
                            "limit_desc": "保留的历史记录最大数量。",
                            "retention": "日志保留天数",
                            "retention_desc": "自动删除旧日志文件前的保留天数。"
                        },
                        "network": {
                            "local_info": "本地网络信息",
                            "local_info_desc": "您的设备网络详情",
                            "hostname": "主机名",
                            "local_ips": "本地IP地址",
                            "lan_devices": "局域网设备",
                            "lan_devices_desc": "发现的网络同步服务器",
                            "scanning": "扫描中...",
                            "rescan": "重新扫描",
                            "no_devices": "未发现局域网设备",
                            "no_devices_hint": "请确保其他设备已启用服务器",
                            "copy_address": "复制地址",
                            "physical_adapter": "物理网卡",
                            "connected_clients": "已连接设备",
                            "connected_clients_desc": "最近连接到本机服务器的设备",
                            "last_active": "最后活跃"
                        },
                        "history": {
                            "recent_items": "最近记录",
                            "clear_all": "清空历史",
                            "confirm_clear": "确定要清空未固定的历史记录吗？",
                            "pin": "固定",
                            "unpin": "取消固定",
                            "delete": "删除",
                            "empty": "未找到剪贴板历史记录",
                            "empty_content": "空内容",
                            "clipboard_image": "剪贴板图片",
                            "open_full_size": "打开完整尺寸",
                            "unknown_file": "未知文件",
                            "copy_to_clipboard": "复制到剪贴板"
                        },
                        "about": {
                            "subtitle": "跨设备剪贴板同步工具",
                            "version": "版本",
                            "tauriVersion": "Tauri 版本",
                            "identifier": "应用标识",
                            "license": "许可证",
                            "project": "项目地址",
                            "update": "更新",
                            "checking": "检查中...",
                            "checkUpdate": "检查更新",
                            "upToDate": "当前已是最新版本！",
                            "checkUpdateFailed": "检查更新失败，请稍后重试",
                            "newVersionAvailable": "发现新版本",
                            "download": "前往下载",
                            "dependencies": "依赖库",
                            "name": "名称",
                            "category": "类别"
                        }
                    }
                }
            }
        },
        fallbackLng: "en",
        interpolation: {
            escapeValue: false
        },
        detection: {
            order: ['localStorage', 'navigator'],
            caches: ['localStorage']
        }
    });

export default i18n;
