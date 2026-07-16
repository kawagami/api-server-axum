"use client";

import { useState } from "react";
import Link from "next/link";
import { Check, ChevronLeft, ChevronRight, Copy, Download, Loader2, RotateCcw, Trash2 } from "lucide-react";
import { AdminHeadRow, AdminRow, AdminTable, AdminTd, AdminTh } from "@/components/admin/table";
import { createTorrentDownloadLinks, deleteTorrent, retryTorrent } from "@/api/torrents";
import StorageBars from "./storage-bars";
import AddTorrentForm from "./add-torrent-form";
import FileDownloadModal, { displayName } from "./file-download-modal";
import { useTorrentLive } from "./useTorrentLive";
import { TORRENT_STATUS_BADGE } from "@/libs/badge-styles";
import { formatBytes } from "@/libs/format-bytes";
import type { Torrent, TorrentStorage } from "@/types";

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

const pageBtnClass = "flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm transition-colors";
const pageBtnDisabledClass = "flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-200 dark:border-neutral-700 text-neutral-300 dark:text-neutral-600 text-sm cursor-not-allowed";

export default function TorrentManager({ initialTorrents, initialTotal, initialStorage, status, page, perPage }: Props) {
    const { torrents, total, storage, liveMap, refresh } = useTorrentLive({
        initialTorrents,
        initialTotal,
        initialStorage,
        status,
        page,
        perPage,
    });

    const [busyId, setBusyId] = useState<number | null>(null);
    const [modalTorrent, setModalTorrent] = useState<Torrent | null>(null);
    const [downloadingKey, setDownloadingKey] = useState<string | null>(null);
    const [copiedKey, setCopiedKey] = useState<string | null>(null);

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
            <AddTorrentForm onAdded={refresh} />

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
                                                    className="p-2 rounded text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/30 transition-colors disabled:opacity-50"
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
                                                    className="p-2 rounded text-neutral-500 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors disabled:opacity-50"
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
                                                    className="p-2 rounded text-yellow-600 dark:text-yellow-400 hover:bg-yellow-50 dark:hover:bg-yellow-900/30 transition-colors disabled:opacity-50"
                                                    title={`重試${t.error ? `（${t.error}）` : ""}`}
                                                >
                                                    {isBusy ? <Loader2 className="w-4 h-4 animate-spin" /> : <RotateCcw className="w-4 h-4" />}
                                                </button>
                                            )}
                                            <button
                                                onClick={() => handleDelete(t)}
                                                disabled={isBusy}
                                                className="p-2 rounded text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors disabled:opacity-50"
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
                <FileDownloadModal
                    torrent={modalTorrent}
                    downloadingKey={downloadingKey}
                    copiedKey={copiedKey}
                    onClose={() => setModalTorrent(null)}
                    onDownload={downloadFile}
                    onCopy={copyLink}
                />
            )}
        </div>
    );
}
