"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { Check, ChevronLeft, ChevronRight, Copy, Download, Loader2, Plus, RotateCcw, Trash2, X } from "lucide-react";
import { AdminHeadRow, AdminRow, AdminTable, AdminTd, AdminTh } from "@/components/admin/table";
import { useWsSubscribe } from "@/hooks/useWsSubscribe";
import {
    createTorrentDownloadLinks,
    deleteTorrent,
    getTorrent,
    getTorrents,
    getTorrentStorage,
    postTorrent,
    retryTorrent,
} from "@/api/torrents";
import StorageBars from "./storage-bars";
import { TORRENT_STATUS_BADGE } from "@/libs/badge-styles";
import { formatBytes } from "@/libs/format-bytes";
import type { Torrent, TorrentProgressEvent, TorrentStorage } from "@/types";

interface Props {
    initialTorrents: Torrent[];
    initialTotal: number;
    initialStorage: TorrentStorage | null;
    status: string;
    page: number;
    perPage: number;
}

function buildHref(status: string, page: number) {
    const params = new URLSearchParams();
    if (status) params.append("status", status);
    if (page > 1) params.append("page", String(page));
    const qs = params.toString();
    return `/admin/torrents${qs ? `?${qs}` : ""}`;
}

function displayName(t: Torrent) {
    if (t.name) return t.name;
    const uri = t.magnet_uri;
    return uri.length > 60 ? `${uri.slice(0, 60)}…` : uri;
}

function addErrorMessage(status?: number, message?: string) {
    switch (status) {
        case 409: return "相同 torrent 已存在";
        case 422: return "magnet 格式錯誤（缺少 btih）";
        case 507: return "伺服器 torrent 容量已滿，請先刪除舊任務";
        default: return message ?? "新增失敗";
    }
}

const pageBtnClass = "flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm transition-colors";
const pageBtnDisabledClass = "flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-200 dark:border-neutral-700 text-neutral-300 dark:text-neutral-600 text-sm cursor-not-allowed";

export default function TorrentManager({ initialTorrents, initialTotal, initialStorage, status, page, perPage }: Props) {
    const [torrents, setTorrents] = useState<Torrent[]>(initialTorrents);
    const [total, setTotal] = useState(initialTotal);
    const [storage, setStorage] = useState<TorrentStorage | null>(initialStorage);
    const [liveMap, setLiveMap] = useState<Record<number, TorrentProgressEvent>>({});

    const [magnet, setMagnet] = useState("");
    const [adding, setAdding] = useState(false);
    const [formError, setFormError] = useState<string | null>(null);

    const [busyId, setBusyId] = useState<number | null>(null);
    const [modalTorrent, setModalTorrent] = useState<Torrent | null>(null);
    const [downloadingKey, setDownloadingKey] = useState<string | null>(null);
    const [copiedKey, setCopiedKey] = useState<string | null>(null);

    // WS「進度有變動才推」，卡住的任務不會再推 — 掛載/刷新後打詳情端點把 live 進度補回來
    const seedLive = useCallback(async (list: Torrent[]) => {
        const ids = list.filter((t) => t.status === "downloading").map((t) => t.id);
        if (ids.length === 0) return;
        const details = await Promise.all(ids.map((id) => getTorrent(id).catch(() => null)));
        setLiveMap((prev) => {
            const next = { ...prev };
            for (const d of details) {
                if (d?.live) next[d.id] = { id: d.id, name: d.name ?? "", ...d.live };
            }
            return next;
        });
    }, []);

    useEffect(() => {
        // setState 發生在 fetch await 之後，非同步、不會 cascading render
        // eslint-disable-next-line react-hooks/set-state-in-effect
        seedLive(initialTorrents);
    }, [seedLive, initialTorrents]);

    const refresh = useCallback(async () => {
        try {
            const [{ data, total }, storageRes] = await Promise.all([
                getTorrents(status || null, page, perPage),
                getTorrentStorage().catch(() => null),
            ]);
            setTorrents(data);
            setTotal(total);
            if (storageRes) setStorage(storageRes);
            seedLive(data);
        } catch {
            // 列表刷新失敗就維持現狀，下次事件再試
        }
    }, [status, page, perPage, seedLive]);

    useWsSubscribe("torrent_progress", (data) => {
        const ev = data as TorrentProgressEvent;
        setLiveMap((prev) => ({ ...prev, [ev.id]: ev }));
        // pending → downloading 的轉換靠進度推播得知，順手更新 status
        setTorrents((prev) =>
            prev.map((t) => (t.id === ev.id && t.status === "pending" ? { ...t, status: "downloading" } : t)),
        );
    });

    useWsSubscribe("torrent_completed", (data) => {
        const ev = data as { id: number };
        setLiveMap((prev) => {
            const next = { ...prev };
            delete next[ev.id];
            return next;
        });
        refresh();
    });

    useWsSubscribe("torrent_failed", (data) => {
        const ev = data as { id: number };
        setLiveMap((prev) => {
            const next = { ...prev };
            delete next[ev.id];
            return next;
        });
        refresh();
    });

    const handleAdd = async (e: React.FormEvent) => {
        e.preventDefault();
        const uri = magnet.trim();
        if (!uri || adding) return;
        setAdding(true);
        setFormError(null);
        const result = await postTorrent(uri);
        setAdding(false);
        if (result.ok) {
            setMagnet("");
            refresh();
        } else {
            setFormError(addErrorMessage(result.status, result.message));
        }
    };

    // 點擊當下才產生連結（效期以回應的 expires_at 為準，後端 torrent_link_ttl_minutes 可調）
    const fetchLink = async (torrentId: number, fileIndex: number): Promise<string | null> => {
        const result = await createTorrentDownloadLinks(torrentId);
        if (!result.ok) {
            alert(`產生下載連結失敗：${result.message}`);
            return null;
        }
        const link = result.links?.find((l) => l.file_index === fileIndex) ?? result.links?.[0];
        return link?.url ?? null;
    };

    // 大檔走瀏覽器原生下載，不用 fetch + blob
    const downloadFile = async (torrentId: number, fileIndex: number) => {
        if (downloadingKey) return;
        setDownloadingKey(`${torrentId}:${fileIndex}`);
        const url = await fetchLink(torrentId, fileIndex);
        setDownloadingKey(null);
        if (!url) return;
        const a = document.createElement("a");
        a.href = url;
        a.click();
    };

    // 複製連結給外部下載器（aria2 / IDM）用 — 一樣現換新 token
    const copyLink = async (torrentId: number, fileIndex: number) => {
        const key = `${torrentId}:${fileIndex}`;
        if (downloadingKey) return;
        setDownloadingKey(key);
        const url = await fetchLink(torrentId, fileIndex);
        setDownloadingKey(null);
        if (!url) return;
        await navigator.clipboard.writeText(url);
        setCopiedKey(key);
        setTimeout(() => setCopiedKey((k) => (k === key ? null : k)), 1500);
    };

    const handleDownload = (t: Torrent) => {
        if (t.files && t.files.length > 1) {
            setModalTorrent(t);
        } else {
            downloadFile(t.id, t.files?.[0]?.index ?? 0);
        }
    };

    const handleRetry = async (t: Torrent) => {
        if (busyId) return;
        setBusyId(t.id);
        const result = await retryTorrent(t.id);
        setBusyId(null);
        if (!result.ok) alert(`重試失敗：${result.message}`);
        refresh();
    };

    const handleDelete = async (t: Torrent) => {
        if (busyId) return;
        if (!confirm(`確定刪除「${displayName(t)}」？伺服器上的檔案會一併刪除。`)) return;
        setBusyId(t.id);
        const result = await deleteTorrent(t.id);
        setBusyId(null);
        if (!result.ok) alert(`刪除失敗：${result.message}`);
        refresh();
    };

    const offset = (page - 1) * perPage;
    const hasPrev = page > 1;
    const hasNext = offset + perPage < total;

    return (
        <div className="flex flex-col gap-4">
            <StorageBars storage={storage} />
            <form onSubmit={handleAdd} className="flex flex-col gap-1">
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={magnet}
                        onChange={(e) => setMagnet(e.target.value)}
                        placeholder="magnet:?xt=urn:btih:..."
                        className="flex-1 px-3 py-2 rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 dark:placeholder-neutral-500 focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                    <button
                        type="submit"
                        disabled={adding || !magnet.trim()}
                        className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white text-sm transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
                    >
                        {adding ? <Loader2 className="w-4 h-4 animate-spin" /> : <Plus className="w-4 h-4" />}
                        新增
                    </button>
                </div>
                {formError && <p className="text-sm text-red-500">{formError}</p>}
            </form>

            <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-x-auto">
                <AdminTable>
                    <thead>
                        <AdminHeadRow>
                            <AdminTh>名稱</AdminTh>
                            <AdminTh>狀態</AdminTh>
                            <AdminTh>大小</AdminTh>
                            <AdminTh className="min-w-44">進度</AdminTh>
                            <AdminTh>建立時間</AdminTh>
                            <AdminTh>操作</AdminTh>
                        </AdminHeadRow>
                    </thead>
                    <tbody>
                        {torrents.length === 0 && (
                            <AdminRow>
                                <AdminTd colSpan={6} className="text-center text-neutral-500 dark:text-neutral-400 py-8">
                                    沒有任務
                                </AdminTd>
                            </AdminRow>
                        )}
                        {torrents.map((t) => {
                            const live = liveMap[t.id];
                            const isBusy = busyId === t.id;
                            return (
                                <AdminRow key={t.id}>
                                    <AdminTd className="max-w-xs">
                                        <span className="block truncate text-sm" title={t.name ?? t.magnet_uri}>
                                            {displayName(t)}
                                        </span>
                                        <span className="block text-xs text-neutral-500 dark:text-neutral-400">
                                            {t.created_by}
                                        </span>
                                    </AdminTd>
                                    <AdminTd>
                                        <span
                                            className={`inline-block px-2 py-0.5 rounded text-xs font-medium ${TORRENT_STATUS_BADGE[t.status]}`}
                                            title={t.status === "failed" ? t.error ?? undefined : undefined}
                                        >
                                            {t.status}
                                        </span>
                                    </AdminTd>
                                    <AdminTd className="whitespace-nowrap text-sm">
                                        {formatBytes(live?.total_bytes ?? t.total_size)}
                                    </AdminTd>
                                    <AdminTd>
                                        {t.status === "downloading" && live ? (
                                            <div className="flex flex-col gap-1">
                                                <div className="h-2 rounded-full bg-neutral-200 dark:bg-neutral-700 overflow-hidden">
                                                    <div
                                                        className="h-full bg-primary-500 rounded-full transition-[width]"
                                                        style={{ width: `${live.progress}%` }}
                                                    />
                                                </div>
                                                <span className="text-xs text-neutral-500 dark:text-neutral-400">
                                                    {live.progress.toFixed(2)}% · {live.down_speed} · {live.peers} peers
                                                </span>
                                            </div>
                                        ) : t.status === "completed" ? (
                                            <span className="text-sm text-green-600 dark:text-green-400">100%</span>
                                        ) : t.status === "failed" ? (
                                            <span className="block text-xs text-red-500 truncate max-w-44" title={t.error ?? undefined}>
                                                {t.error ?? "失敗"}
                                            </span>
                                        ) : (
                                            <span className="text-sm text-neutral-400">—</span>
                                        )}
                                    </AdminTd>
                                    <AdminTd className="whitespace-nowrap text-sm">
                                        {new Date(t.created_at).toLocaleString()}
                                    </AdminTd>
                                    <AdminTd>
                                        <div className="flex items-center gap-2">
                                            {t.status === "completed" && (
                                                <button
                                                    onClick={() => handleDownload(t)}
                                                    disabled={downloadingKey !== null}
                                                    className="p-1.5 rounded text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/30 transition-colors disabled:opacity-50"
                                                    title="下載"
                                                >
                                                    {downloadingKey?.startsWith(`${t.id}:`) && !modalTorrent
                                                        ? <Loader2 className="w-4 h-4 animate-spin" />
                                                        : <Download className="w-4 h-4" />}
                                                </button>
                                            )}
                                            {t.status === "completed" && !(t.files && t.files.length > 1) && (
                                                <button
                                                    onClick={() => copyLink(t.id, t.files?.[0]?.index ?? 0)}
                                                    disabled={downloadingKey !== null}
                                                    className="p-1.5 rounded text-neutral-500 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors disabled:opacity-50"
                                                    title="複製下載連結"
                                                >
                                                    {copiedKey === `${t.id}:${t.files?.[0]?.index ?? 0}`
                                                        ? <Check className="w-4 h-4 text-green-500" />
                                                        : <Copy className="w-4 h-4" />}
                                                </button>
                                            )}
                                            {t.status === "failed" && (
                                                <button
                                                    onClick={() => handleRetry(t)}
                                                    disabled={isBusy}
                                                    className="p-1.5 rounded text-yellow-600 dark:text-yellow-400 hover:bg-yellow-50 dark:hover:bg-yellow-900/30 transition-colors disabled:opacity-50"
                                                    title={`重試${t.error ? `（${t.error}）` : ""}`}
                                                >
                                                    {isBusy ? <Loader2 className="w-4 h-4 animate-spin" /> : <RotateCcw className="w-4 h-4" />}
                                                </button>
                                            )}
                                            <button
                                                onClick={() => handleDelete(t)}
                                                disabled={isBusy}
                                                className="p-1.5 rounded text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors disabled:opacity-50"
                                                title="刪除"
                                            >
                                                {isBusy ? <Loader2 className="w-4 h-4 animate-spin" /> : <Trash2 className="w-4 h-4" />}
                                            </button>
                                        </div>
                                    </AdminTd>
                                </AdminRow>
                            );
                        })}
                    </tbody>
                </AdminTable>
            </div>

            <div className="flex items-center justify-between">
                <span className="text-sm text-neutral-500 dark:text-neutral-400">
                    {total > 0 ? `${offset + 1}–${Math.min(offset + perPage, total)} / ${total}` : "0 / 0"}
                </span>
                <div className="flex gap-2">
                    {hasPrev ? (
                        <Link href={buildHref(status, page - 1)} className={pageBtnClass}>
                            <ChevronLeft className="w-4 h-4" /> 上一頁
                        </Link>
                    ) : (
                        <span className={pageBtnDisabledClass}>
                            <ChevronLeft className="w-4 h-4" /> 上一頁
                        </span>
                    )}
                    {hasNext ? (
                        <Link href={buildHref(status, page + 1)} className={pageBtnClass}>
                            下一頁 <ChevronRight className="w-4 h-4" />
                        </Link>
                    ) : (
                        <span className={pageBtnDisabledClass}>
                            下一頁 <ChevronRight className="w-4 h-4" />
                        </span>
                    )}
                </div>
            </div>

            {modalTorrent && (
                <div
                    className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4"
                    onClick={() => setModalTorrent(null)}
                >
                    <div
                        className="w-full max-w-lg max-h-[80svh] overflow-auto bg-white dark:bg-neutral-900 rounded-lg shadow-xl"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="flex items-center justify-between px-4 py-3 border-b border-neutral-200 dark:border-neutral-700">
                            <span className="font-semibold text-neutral-800 dark:text-white truncate" title={modalTorrent.name ?? undefined}>
                                {displayName(modalTorrent)}
                            </span>
                            <button
                                onClick={() => setModalTorrent(null)}
                                className="p-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                                aria-label="關閉"
                            >
                                <X className="w-5 h-5 text-neutral-500" />
                            </button>
                        </div>
                        <ul className="divide-y divide-neutral-100 dark:divide-neutral-800">
                            {(modalTorrent.files ?? []).map((f) => {
                                const key = `${modalTorrent.id}:${f.index}`;
                                return (
                                    <li key={f.index} className="flex items-center gap-3 px-4 py-2.5">
                                        <div className="flex-1 min-w-0">
                                            <span className="block text-sm text-neutral-800 dark:text-neutral-200 truncate" title={f.path}>
                                                {f.path}
                                            </span>
                                            <span className="text-xs text-neutral-500 dark:text-neutral-400">
                                                {formatBytes(f.size)}
                                            </span>
                                        </div>
                                        <button
                                            onClick={() => downloadFile(modalTorrent.id, f.index)}
                                            disabled={downloadingKey !== null}
                                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary-500 hover:bg-primary-600 text-white text-sm transition-colors disabled:opacity-60"
                                        >
                                            {downloadingKey === key
                                                ? <Loader2 className="w-4 h-4 animate-spin" />
                                                : <Download className="w-4 h-4" />}
                                            下載
                                        </button>
                                        <button
                                            onClick={() => copyLink(modalTorrent.id, f.index)}
                                            disabled={downloadingKey !== null}
                                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-neutral-300 dark:border-neutral-600 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-sm transition-colors disabled:opacity-60"
                                            title="複製下載連結"
                                        >
                                            {copiedKey === key
                                                ? <Check className="w-4 h-4 text-green-500" />
                                                : <Copy className="w-4 h-4" />}
                                            複製
                                        </button>
                                    </li>
                                );
                            })}
                        </ul>
                    </div>
                </div>
            )}
        </div>
    );
}
