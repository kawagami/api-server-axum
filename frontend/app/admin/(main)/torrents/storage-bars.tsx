import { formatBytes } from "@/libs/format-bytes";
import type { TorrentStorage } from "@/types";

function barColor(pct: number) {
    if (pct >= 95) return "bg-red-500";
    if (pct >= 80) return "bg-yellow-500";
    return "bg-primary-500";
}

function UsageBar({ label, used, total }: { label: string; used: number; total: number }) {
    const pct = total > 0 ? Math.min((used / total) * 100, 100) : 0;
    return (
        <div className="flex-1 min-w-56">
            <div className="flex justify-between text-xs text-neutral-500 dark:text-neutral-400 mb-1">
                <span>{label}</span>
                <span>
                    {formatBytes(used)} / {formatBytes(total)}（{pct.toFixed(1)}%）
                </span>
            </div>
            <div className="h-2 rounded-full bg-neutral-200 dark:bg-neutral-700 overflow-hidden">
                <div className={`h-full rounded-full ${barColor(pct)}`} style={{ width: `${pct}%` }} />
            </div>
        </div>
    );
}

export default function StorageBars({ storage }: { storage: TorrentStorage | null }) {
    if (!storage) return null;
    return (
        <div className="flex flex-wrap gap-x-6 gap-y-3">
            <UsageBar
                label="磁碟"
                used={storage.disk.total_bytes - storage.disk.available_bytes}
                total={storage.disk.total_bytes}
            />
            <UsageBar label="Torrent 配額" used={storage.torrent.used_bytes} total={storage.torrent.max_bytes} />
        </div>
    );
}
