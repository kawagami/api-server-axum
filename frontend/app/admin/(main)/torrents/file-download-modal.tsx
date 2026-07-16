"use client";

import { Check, Copy, Download, Loader2, X } from "lucide-react";
import { formatBytes } from "@/libs/format-bytes";
import type { Torrent } from "@/types";

export function displayName(t: Torrent) {
    if (t.name) return t.name;
    const uri = t.magnet_uri;
    return uri.length > 60 ? `${uri.slice(0, 60)}…` : uri;
}

interface Props {
    torrent: Torrent;
    downloadingKey: string | null;
    copiedKey: string | null;
    onClose: () => void;
    onDownload: (torrentId: number, fileIndex: number) => void;
    onCopy: (torrentId: number, fileIndex: number) => void;
}

export default function FileDownloadModal({ torrent, downloadingKey, copiedKey, onClose, onDownload, onCopy }: Props) {
    return (
        <div
            className="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4"
            onClick={onClose}
        >
            <div
                className="w-full max-w-lg max-h-[80svh] overflow-auto bg-white dark:bg-neutral-900 rounded-lg shadow-xl"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between px-4 py-3 border-b border-neutral-200 dark:border-neutral-700">
                    <span className="font-semibold text-neutral-800 dark:text-white truncate" title={torrent.name ?? undefined}>
                        {displayName(torrent)}
                    </span>
                    <button
                        onClick={onClose}
                        className="p-1 rounded hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                        aria-label="關閉"
                    >
                        <X className="w-5 h-5 text-neutral-500" />
                    </button>
                </div>
                <ul className="divide-y divide-neutral-100 dark:divide-neutral-800">
                    {(torrent.files ?? []).map((f) => {
                        const key = `${torrent.id}:${f.index}`;
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
                                    onClick={() => onDownload(torrent.id, f.index)}
                                    disabled={downloadingKey !== null}
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-primary-500 hover:bg-primary-600 text-white text-sm transition-colors disabled:opacity-60"
                                >
                                    {downloadingKey === key
                                        ? <Loader2 className="w-4 h-4 animate-spin" />
                                        : <Download className="w-4 h-4" />}
                                    <span className="hidden sm:inline">下載</span>
                                </button>
                                <button
                                    onClick={() => onCopy(torrent.id, f.index)}
                                    disabled={downloadingKey !== null}
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-neutral-300 dark:border-neutral-600 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 text-sm transition-colors disabled:opacity-60"
                                    title="複製下載連結"
                                >
                                    {copiedKey === key
                                        ? <Check className="w-4 h-4 text-green-500" />
                                        : <Copy className="w-4 h-4" />}
                                    <span className="hidden sm:inline">複製</span>
                                </button>
                            </li>
                        );
                    })}
                </ul>
            </div>
        </div>
    );
}
