export interface ServerConfig {
    host: string;
    port: number;
    tls?: {
        cert: string;
        key: string;
    } | null;
    enabled: boolean;
}

export interface ClientConfig {
    enabled: boolean;
    remote_host: string;
    remote_port: number;
}

export interface AuthConfig {
    token?: string | null;
    encrypt_password?: string | null;
}

export interface HistoryConfig {
    max_count: number;
    log_retention_days: number;
}

export interface GeneralConfig {
    device_name: string;
}

export interface Config {
    server: ServerConfig;
    client: ClientConfig;
    auth: AuthConfig;
    history: HistoryConfig;
    general: GeneralConfig;
}

export interface HistoryItem {
    id: number;
    type: "Text" | "Image" | "File";
    content?: string;
    html?: string;
    file?: string;
    device?: string;
    timestamp: string;
    pinned?: boolean;
}
