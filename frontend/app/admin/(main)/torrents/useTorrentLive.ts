"use client";

import { useCallback, useEffect, useState } from "react";
import { useWsSubscribe } from "@/hooks/useWsSubscribe";
import { getTorrent, getTorrents, getTorrentStorage } from "@/api/torrents";
import type { Torrent, TorrentProgressEvent, TorrentStorage } from "@/types";

interface Args {
    initialTorrents: Torrent[];
    initialTotal: number;
    initialStorage: TorrentStorage | null;
    status: string;
    page: number;
    perPage: number;
}

// 任務列表狀態 + WS 即時進度：progress 推播進 liveMap、completed/failed 清 liveMap 並刷新列表；
// completed/failed 的 refresh 會回頭 seedLive，所以列表狀態與 refresh 一併收在這個 hook
export function useTorrentLive({ initialTorrents, initialTotal, initialStorage, status, page, perPage }: Args) {
    const [torrents, setTorrents] = useState<Torrent[]>(initialTorrents);
    const [total, setTotal] = useState(initialTotal);
    const [storage, setStorage] = useState<TorrentStorage | null>(initialStorage);
    const [liveMap, setLiveMap] = useState<Record<number, TorrentProgressEvent>>({});

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

    return { torrents, total, storage, liveMap, refresh };
}
