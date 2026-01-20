import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { HistoryItem, Config } from '../types';
import { FileText, Image, File, Copy, Check, ExternalLink, Trash2, Pin, PinOff, XCircle } from 'lucide-react';
import { cn } from '../lib/utils';
import { useTranslation } from 'react-i18next';
import { Button } from "./ui/button";

function History() {
    const { t } = useTranslation();
    const [history, setHistory] = useState<HistoryItem[]>([]);
    const [copiedId, setCopiedId] = useState<number | null>(null);
    const [config, setConfig] = useState<Config | null>(null);

    // Fetch config to get server host/port for image preview
    useEffect(() => {
        invoke<Config>('get_config').then(setConfig).catch(console.error);
    }, []);

    const fetchHistory = async () => {
        try {
            const items = await invoke<HistoryItem[]>('get_history');
            setHistory(items.sort((a, b) => b.id - a.id));
        } catch (err) {
            console.error("Failed to fetch history:", err);
        }
    };

    useEffect(() => {
        fetchHistory();
        const interval = setInterval(fetchHistory, 2000);
        return () => clearInterval(interval);
    }, []);

    const handleCopy = (item: HistoryItem) => {
        if (item.type === 'Text' && item.content) {
            navigator.clipboard.writeText(item.content)
                .then(() => {
                    setCopiedId(item.id);
                    setTimeout(() => setCopiedId(null), 2000);
                })
                .catch(err => console.error("Copy failed", err));
        }
    };

    const handleDelete = async (id: number) => {
        try {
            await invoke('delete_history_item', { id });
            fetchHistory();
        } catch (err) {
            console.error("Failed to delete item:", err);
        }
    };

    const handleClear = async () => {
        if (window.confirm(t('history.confirm_clear', 'Are you sure you want to clear all history? Pinned items will optionally be deleted (checking logic). Actually current implementation deletes Unpinned only logic? No, backend deletes unpinned only.'))) {
            // Wait, backend implementation: DELETE FROM history WHERE pinned = 0.
            // So verify translation says "Clear Unpinned" or "Clear History".
            try {
                await invoke('clear_history');
                fetchHistory();
            } catch (err) {
                console.error("Failed to clear history:", err);
            }
        }
    };

    const handleTogglePin = async (id: number) => {
        try {
            await invoke('toggle_pin', { id });
            fetchHistory();
        } catch (err) {
            console.error("Failed to toggle pin:", err);
        }
    };

    const formatTimestamp = (ts: string) => {
        // Create date object from timestamp string
        // Assuming timestamp is in a parseable format (like ISO or YYYY-MM-DD HH:mm:ss)
        // If rust sends formatted string, just return it.
        return ts;
    };

    const IconForType = ({ type }: { type: string }) => {
        switch (type) {
            case 'Image': return <Image size={16} className="text-purple-500" />;
            case 'File': return <File size={16} className="text-orange-500" />;
            default: return <FileText size={16} className="text-blue-500" />;
        }
    };

    const EmptyState = () => (
        <div className="flex flex-col items-center justify-center h-full text-muted-foreground opacity-60">
            <div className="bg-muted p-6 rounded-full mb-4 border border-border">
                <FileText size={32} className="opacity-50" />
            </div>
            <p className="font-medium">{t('history.empty', 'No clipboard history found')}</p>
        </div>
    );


    if (history.length === 0) {
        return <EmptyState />;
    }

    // Construct Image URL
    const getImageUrl = (filename: string) => {
        if (!config) return '';
        const host = config.server.host === '0.0.0.0' ? '127.0.0.1' : config.server.host;
        return `http://${host}:${config.server.port}/file/${filename}`;
    };

    return (
        <div className="h-full flex flex-col gap-4">
            <div className="flex justify-between items-center px-1">
                <h3 className="font-medium text-sm text-muted-foreground">{t('history.recent_items', 'Recent Items')}</h3>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleClear}
                    className="text-destructive hover:text-destructive hover:bg-destructive/10 h-8 px-2"
                >
                    <Trash2 size={14} className="mr-1.5" />
                    {t('history.clear_all', 'Clear History')}
                </Button>
            </div>

            <div className="space-y-3 pb-8 overflow-y-auto pr-1">
                {history.map((item) => (
                    <div
                        key={item.id}
                        className={cn(
                            "group relative flex flex-col gap-2 p-3 rounded-lg border transition-all animate-in fade-in slide-in-from-bottom-2 duration-300",
                            "hover:shadow-md bg-card/50 hover:bg-card",
                            item.pinned ? "border-primary/50 bg-primary/5" : "border-border/50"
                        )}
                    >
                        {/* Header Row */}
                        <div className="flex items-center justify-between gap-2">
                            <div className="flex items-center gap-2">
                                <div className={cn("p-1.5 rounded-md bg-secondary/50", item.pinned && "bg-primary/10")}>
                                    <IconForType type={item.type} />
                                </div>
                                <span className={cn("text-xs font-semibold px-2 py-0.5 rounded-full border",
                                    item.pinned ? "bg-primary/10 border-primary/20 text-primary" : "bg-secondary border-border text-muted-foreground"
                                )}>
                                    {item.type}
                                </span>
                                {item.device && (
                                    <span className="text-[10px] bg-secondary px-1.5 py-0.5 rounded text-muted-foreground border border-border flex items-center gap-1">
                                        💻 {item.device}
                                    </span>
                                )}
                                <span className="text-[10px] text-muted-foreground font-mono">{formatTimestamp(item.timestamp)}</span>
                            </div>

                            <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className="h-7 w-7 text-muted-foreground hover:text-primary"
                                    onClick={() => handleCopy(item)}
                                    title={t('history.copy_to_clipboard', 'Copy to Clipboard')}
                                >
                                    {copiedId === item.id ? <Check size={14} className="text-green-500" /> : <Copy size={14} />}
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className={cn("h-7 w-7", item.pinned ? "text-primary opacity-100" : "text-muted-foreground")}
                                    onClick={() => handleTogglePin(item.id)}
                                    title={item.pinned ? t('history.unpin', 'Unpin') : t('history.pin', 'Pin')}
                                >
                                    {item.pinned ? <PinOff size={14} /> : <Pin size={14} />}
                                </Button>
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className="h-7 w-7 text-destructive/70 hover:text-destructive hover:bg-destructive/10"
                                    onClick={() => handleDelete(item.id)}
                                    title={t('history.delete', 'Delete')}
                                >
                                    <XCircle size={14} />
                                </Button>
                            </div>
                        </div>

                        {item.type === 'Text' ? (
                            <p className="text-sm font-mono break-words leading-relaxed selection:bg-primary/20 line-clamp-[10]">
                                {item.content || <span className="text-muted-foreground italic">{t('history.empty_content', 'Empty Content')}</span>}
                            </p>
                        ) : item.type === 'Image' && item.file ? (
                            <div className="mt-2 relative group/image max-w-sm">
                                <div className="rounded-md border border-border overflow-hidden bg-muted/50">
                                    <img
                                        src={getImageUrl(item.file)}
                                        alt={t('history.clipboard_image', 'Clipboard Image')}
                                        className="max-h-60 w-auto object-contain bg-[url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI4IiBoZWlnaHQ9IjgiPjxyZWN0IHdpZHRoPSI4IiBoZWlnaHQ9IjgiIGZpbGw9IiNmZmZmZmYiLz48cGF0aCBkPSJNMCAwTDggOFpNOCAwTDAgOFoiIHN0cm9rZT0iI2IyYjJiMiIgc3Ryb2tlLXdpZHRoPSIxIi8+PC9zdmc+')] bg-repeat"
                                        onError={(e) => {
                                            (e.currentTarget as HTMLImageElement).style.display = 'none';
                                            (e.currentTarget.parentElement as HTMLElement).classList.add('hidden');
                                        }}
                                        loading="lazy"
                                    />
                                </div>
                                <a
                                    href={getImageUrl(item.file)}
                                    target="_blank"
                                    rel="noreferrer"
                                    className="absolute top-2 right-2 bg-black/60 text-white p-1.5 rounded opacity-0 group-hover/image:opacity-100 transition-opacity hover:bg-black/80"
                                    title={t('history.open_full_size', 'Open full size')}
                                >
                                    <ExternalLink size={12} />
                                </a>
                            </div>
                        ) : (
                            <div className="text-sm text-muted-foreground italic bg-secondary/50 p-2 rounded border border-dashed border-border mt-1">
                                <span className="flex items-center gap-2 break-all">
                                    <File size={14} /> {item.file || item.content || t('history.unknown_file', 'Unknown File')}
                                </span>
                            </div>
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}

export default History;
