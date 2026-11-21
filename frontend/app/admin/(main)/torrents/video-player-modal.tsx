"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Loader2, X } from "lucide-react";
import Modal from "@/components/modal";
import { createTorrentDownloadLinks } from "@/api/torrents";
import type { TorrentFile } from "@/types";

/** 瀏覽器「可能」播得動的容器；實際能不能播還要看編碼（H.264/AAC 最穩，HEVC/AC3 看瀏覽器） */
const VIDEO_EXTENSIONS = ["mp4", "m4v", "webm", "mkv", "mov", "ogv"];

export function isVideoFile(path: string) {
    const ext = path.split(".").pop()?.toLowerCase() ?? "";
    return VIDEO_EXTENSIONS.includes(ext);
}

/** 播放中連結過期（torrent_link_ttl_minutes）時自動換新 token 的次數上限，防止真正壞檔時無限重試 */
const MAX_REFRESH = 3;

interface Props {
    torrentId: number;
    file: TorrentFile;
    onClose: () => void;
}

export default function VideoPlayerModal({ torrentId, file, onClose }: Props) {
    const videoRef = useRef<HTMLVideoElement>(null);
    const [src, setSrc] = useState<string | null>(null);
    const [error, setError] = useState<string | null>(null);
    // 是否曾成功讀到 metadata：讀過之後才出錯 = 連結過期 / 網路中斷，沒讀過 = 格式不支援
    const loadedRef = useRef(false);
    const resumeAtRef = useRef(0);
    const refreshCountRef = useRef(0);

    const fetchSrc = useCallback(async () => {
        const result = await createTorrentDownloadLinks(torrentId);
        const link = result.ok ? result.links?.find((l) => l.file_index === file.index) : undefined;
        if (!link) {
            setError(`產生播放連結失敗：${result.message ?? "找不到檔案"}`);
            return;
        }
        setSrc(`${link.url}&inline=true`);
    }, [torrentId, file.index]);

    useEffect(() => {
        // 掛載時抓第一次連結（server action，非同步 setState）
        // eslint-disable-next-line react-hooks/set-state-in-effect
        void fetchSrc();
    }, [fetchSrc]);

    const handleLoadedMetadata = () => {
        loadedRef.current = true;
        const video = videoRef.current;
        if (video && resumeAtRef.current > 0) {
            video.currentTime = resumeAtRef.current;
            resumeAtRef.current = 0;
            void video.play().catch(() => {});
        }
    };

    const handleError = () => {
        const video = videoRef.current;
        if (loadedRef.current && refreshCountRef.current < MAX_REFRESH) {
            // 播到一半斷掉多半是 token 過期：換新連結、從原位置接著播
            refreshCountRef.current += 1;
            resumeAtRef.current = video?.currentTime ?? 0;
            void fetchSrc();
            return;
        }
        setError(
            loadedRef.current
                ? "播放中斷，請重新開啟。"
                : "瀏覽器無法播放此檔（常見原因：HEVC 編碼或 AC3/DTS 音軌），請下載後用 VLC 等播放器開啟。",
        );
    };

    return (
        <Modal label={`播放 ${file.path}`} onClose={onClose} size="2xl" backdrop="blur" className="overflow-hidden">
            <div className="flex items-center justify-between gap-3 px-4 py-3 border-b border-neutral-200 dark:border-neutral-700">
                <span className="font-semibold text-neutral-800 dark:text-white truncate" title={file.path}>
                    {file.path}
                </span>
                <button
                    onClick={onClose}
                    className="p-1 rounded-sm hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                    aria-label="關閉"
                >
                    <X className="w-5 h-5 text-neutral-500" />
                </button>
            </div>
            <div className="bg-black aspect-video flex items-center justify-center">
                {error ? (
                    <p className="px-6 text-center text-sm text-neutral-300">{error}</p>
                ) : src ? (
                    <video
                        ref={videoRef}
                        src={src}
                        controls
                        autoPlay
                        playsInline
                        preload="metadata"
                        className="w-full h-full"
                        onLoadedMetadata={handleLoadedMetadata}
                        onError={handleError}
                    />
                ) : (
                    <Loader2 className="w-8 h-8 animate-spin text-neutral-400" />
                )}
            </div>
        </Modal>
    );
}
