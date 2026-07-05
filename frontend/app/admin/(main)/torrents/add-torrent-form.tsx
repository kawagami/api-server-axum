"use client";

import { useState } from "react";
import { Loader2, Plus } from "lucide-react";
import { postTorrent } from "@/api/torrents";

function addErrorMessage(status?: number, message?: string) {
    switch (status) {
        case 409: return "相同 torrent 已存在";
        case 422: return "magnet 格式錯誤（缺少 btih）";
        case 507: return "伺服器 torrent 容量已滿，請先刪除舊任務";
        default: return message ?? "新增失敗";
    }
}

interface Props {
    onAdded: () => void;
}

export default function AddTorrentForm({ onAdded }: Props) {
    const [magnet, setMagnet] = useState("");
    const [adding, setAdding] = useState(false);
    const [formError, setFormError] = useState<string | null>(null);

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
            onAdded();
        } else {
            setFormError(addErrorMessage(result.status, result.message));
        }
    };

    return (
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
    );
}
