"use client";

import { useCallback, useEffect, useState } from "react";
import { Loader2, RefreshCw } from "lucide-react";
import { getGamesOverview } from "@/api/games";
import type { GameOverview } from "@/types";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";

const POLL_INTERVAL_MS = 7000;

// 遊戲代號對照（後端決定清單，查不到 fallback 顯示原字串）
const GAME_LABELS: Record<string, { name: string; type: string }> = {
    chess: { name: "象棋", type: "2 人回合制" },
    gomoku: { name: "五子棋", type: "2 人回合制" },
    banqi: { name: "暗棋翻棋", type: "2 人回合制" },
    western_chess: { name: "西洋棋", type: "2 人回合制" },
    go: { name: "圍棋", type: "2 人回合制" },
    avalon: { name: "阿瓦隆", type: "N 人社交推理" },
    farm: { name: "農場經營", type: "N 人 worker-placement" },
};

function isIdle(g: GameOverview): boolean {
    return g.waiting === 0 && g.playing === 0 && g.players_in_game === 0 && g.queued === 0 && g.lobby === 0;
}

export default function GamesOverview({ initial }: { initial: GameOverview[] }) {
    const [rows, setRows] = useState<GameOverview[]>(initial);
    const [refreshing, setRefreshing] = useState(false);
    const [autoRefresh, setAutoRefresh] = useState(true);
    const [error, setError] = useState<string | null>(null);

    const refresh = useCallback(async () => {
        setRefreshing(true);
        try {
            const data = await getGamesOverview();
            setRows(data);
            setError(null);
        } catch {
            setError("讀取失敗，稍後重試");
        } finally {
            setRefreshing(false);
        }
    }, []);

    useEffect(() => {
        if (!autoRefresh) return;
        const id = setInterval(refresh, POLL_INTERVAL_MS);
        return () => clearInterval(id);
    }, [autoRefresh, refresh]);

    const totalInGame = rows.reduce((sum, g) => sum + g.players_in_game, 0);

    return (
        <div className="flex flex-col gap-4">
            <div className="flex items-center justify-between flex-wrap gap-3">
                <h2 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">
                    對局總覽
                </h2>
                <div className="flex items-center gap-4">
                    <label className="flex items-center gap-2 text-sm text-neutral-600 dark:text-neutral-400 cursor-pointer select-none">
                        <input
                            type="checkbox"
                            checked={autoRefresh}
                            onChange={(e) => setAutoRefresh(e.target.checked)}
                            className="accent-primary-600"
                        />
                        自動更新（每 {POLL_INTERVAL_MS / 1000} 秒）
                    </label>
                    <button
                        onClick={refresh}
                        disabled={refreshing}
                        className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white text-sm font-medium rounded transition-colors"
                    >
                        {refreshing
                            ? <Loader2 className="w-4 h-4 animate-spin" />
                            : <RefreshCw className="w-4 h-4" />}
                        重新整理
                    </button>
                </div>
            </div>

            <div className="text-sm text-neutral-600 dark:text-neutral-400">
                目前在對局中的人數：
                <span className="font-semibold text-primary-600 dark:text-primary-400 ml-1">
                    {totalInGame}
                </span>
                {error && <span className="ml-3 text-red-500">{error}</span>}
            </div>

            <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                <div className="overflow-x-auto">
                <AdminTable>
                    <thead>
                        <AdminHeadRow>
                            <AdminTh>遊戲</AdminTh>
                            <AdminTh>類型</AdminTh>
                            <AdminTh>進行中（桌）</AdminTh>
                            <AdminTh>對局人數</AdminTh>
                            <AdminTh>等待中（桌）</AdminTh>
                            <AdminTh>排隊</AdminTh>
                            <AdminTh>大廳</AdminTh>
                        </AdminHeadRow>
                    </thead>
                    <tbody>
                        {rows.length === 0 ? (
                            <tr>
                                <td colSpan={7} className="border border-neutral-300 dark:border-neutral-700 px-4 py-4 text-center text-neutral-500 dark:text-neutral-400">
                                    無資料
                                </td>
                            </tr>
                        ) : (
                            rows.map((g) => {
                                const meta = GAME_LABELS[g.game];
                                const idle = isIdle(g);
                                const dim = idle ? "text-neutral-400 dark:text-neutral-600" : "";
                                return (
                                    <AdminRow key={g.game}>
                                        <AdminTd className={`font-medium ${dim}`}>
                                            {meta?.name ?? g.game}
                                        </AdminTd>
                                        <AdminTd className={`text-sm ${dim || "text-neutral-500 dark:text-neutral-400"}`}>
                                            {meta?.type ?? "—"}
                                        </AdminTd>
                                        <AdminTd className={dim}>{g.playing}</AdminTd>
                                        <AdminTd className={dim}>{g.players_in_game}</AdminTd>
                                        <AdminTd className={dim}>{g.waiting}</AdminTd>
                                        <AdminTd className={dim}>
                                            {meta?.type.startsWith("N 人") ? "—" : g.queued}
                                        </AdminTd>
                                        <AdminTd className={dim}>{g.lobby}</AdminTd>
                                    </AdminRow>
                                );
                            })
                        )}
                    </tbody>
                </AdminTable>
                </div>
            </div>

            <p className="text-xs text-neutral-400 dark:text-neutral-500">
                數字為記憶體即時快照，伺服器重啟後歸零；匿名對戰，無玩家身份 / 盤面。
            </p>
        </div>
    );
}
