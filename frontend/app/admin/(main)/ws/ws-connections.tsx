"use client";

import { useCallback, useEffect, useState } from "react";
import { Check, Copy, Loader2, RefreshCw, Send } from "lucide-react";
import { getWsConnections } from "@/api/ws";
import { useWsContext } from "@/libs/ws-context";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import PageHeader from "@/components/admin/page-header";
import usePolling from "@/hooks/usePolling";
import { formatDateTime, formatTimeOfDay } from "@/libs/admin-datetime";
import SaySomethingForm from "./say-something-form";
import type { WsConnection, WsUserEventData } from "@/types";

// 輪詢是對帳用的下限，即時增減靠 user_joined / user_left 事件
const POLL_INTERVAL_MS = 7000;
// 連線時長每秒重算一次
const TICK_INTERVAL_MS = 1000;

// 與後端 list_connections 同一套排序：新連線在前，同時間用 addr 破平手。
// connected_at 是固定寬度 ISO 字串，字典序 == 時間序。
function sortRows(rows: WsConnection[]): WsConnection[] {
    return [...rows].sort(
        (a, b) => b.connected_at.localeCompare(a.connected_at) || a.addr.localeCompare(b.addr),
    );
}

function formatDuration(ms: number): string {
    const s = Math.max(0, Math.floor(ms / 1000));
    if (s < 60) return `${s} 秒`;
    const m = Math.floor(s / 60);
    if (m < 60) return `${m} 分 ${s % 60} 秒`;
    const h = Math.floor(m / 60);
    if (h < 24) return `${h} 小時 ${m % 60} 分`;
    return `${Math.floor(h / 24)} 天 ${h % 24} 小時`;
}

// UA 原字串太長塞不進表格，簡化成「OS · 瀏覽器」；完整字串放 title
function deviceLabel(ua: string): string {
    if (!ua || ua === "Unknown browser") return "未知";
    if (/bot|crawler|spider|curl|wget|python-requests|headlesschrome/i.test(ua)) return "Bot / 工具";
    const os = /iPhone|iPad|iPod/i.test(ua) ? "iOS"
        : /Android/i.test(ua) ? "Android"
        : /Mac OS X/i.test(ua) ? "macOS"
        : /Windows/i.test(ua) ? "Windows"
        : /Linux/i.test(ua) ? "Linux"
        : "其他";
    const browser = /Edg\//i.test(ua) ? "Edge"
        : /OPR\//i.test(ua) ? "Opera"
        : /Chrome\//i.test(ua) ? "Chrome"
        : /Firefox\//i.test(ua) ? "Firefox"
        : /Safari\//i.test(ua) ? "Safari"
        : "";
    return browser ? `${os} · ${browser}` : os;
}

function CopyButton({ value, label }: { value: string; label: string }) {
    const [copied, setCopied] = useState(false);

    return (
        <button
            type="button"
            title={`複製${label}`}
            aria-label={`複製${label}`}
            onClick={async () => {
                try {
                    await navigator.clipboard.writeText(value);
                    setCopied(true);
                    setTimeout(() => setCopied(false), 1500);
                } catch {
                    // 非 https 或瀏覽器不給剪貼簿權限，忽略
                }
            }}
            className="shrink-0 p-1 rounded-sm text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
        >
            {copied
                ? <Check className="w-3.5 h-3.5 text-green-600 dark:text-green-400" />
                : <Copy className="w-3.5 h-3.5" />}
        </button>
    );
}

export default function WsConnections({ initial }: { initial: WsConnection[] }) {
    const { subscribe, unsubscribe, onReconnect } = useWsContext();
    const [rows, setRows] = useState<WsConnection[]>(initial);
    const [autoRefresh, setAutoRefresh] = useState(true);
    const [refreshing, setRefreshing] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [onlyLoggedIn, setOnlyLoggedIn] = useState(false);
    const [selectedAddr, setSelectedAddr] = useState<string | null>(null);
    const [focusToken, setFocusToken] = useState(0);
    // 時間相關的值一律 mount 後才填，避免 SSR 與 client 算出不同字串造成 hydration 不一致
    const [now, setNow] = useState<number | null>(null);
    const [lastUpdated, setLastUpdated] = useState<number | null>(null);

    const refresh = useCallback(async () => {
        setRefreshing(true);
        try {
            const data = await getWsConnections();
            setRows(sortRows(data));
            setError(null);
            setLastUpdated(Date.now());
        } catch {
            setError("讀取失敗，稍後重試");
        } finally {
            setRefreshing(false);
        }
    }, []);

    useEffect(() => {
        // 首次取樣放進 timer callback（不在 effect body 同步 setState），
        // initial 是這次頁面 render 時抓的，把它當第一次更新時間
        const first = setTimeout(() => {
            setNow(Date.now());
            setLastUpdated(Date.now());
        }, 0);
        const tick = setInterval(() => setNow(Date.now()), TICK_INTERVAL_MS);
        return () => {
            clearTimeout(first);
            clearInterval(tick);
        };
    }, []);

    // 分頁在背景時不打，切回來若已過期就補一次（連線增減本來就靠 WS 事件即時反映）
    usePolling(refresh, POLL_INTERVAL_MS, autoRefresh);

    // 即時增減：後端 broadcast_to_admins 只推給已登入連線，本頁 admin 收得到
    useEffect(() => {
        const onJoined = (data: unknown) => {
            const d = data as WsUserEventData | null;
            if (!d?.addr) return;
            setRows((prev) => sortRows([
                ...prev.filter((r) => r.addr !== d.addr),
                {
                    addr: d.addr,
                    user_email: d.user_email ?? null,
                    real_ip: d.real_ip ?? "",
                    connected_at: d.connected_at ?? new Date().toISOString(),
                    user_agent: d.user_agent ?? "",
                },
            ]));
            setLastUpdated(Date.now());
        };
        const onLeft = (data: unknown) => {
            const d = data as WsUserEventData | null;
            if (!d?.addr) return;
            setRows((prev) => prev.filter((r) => r.addr !== d.addr));
            setLastUpdated(Date.now());
        };

        subscribe("user_joined", onJoined);
        subscribe("user_left", onLeft);
        // server 重啟會清掉所有連線記錄，重連後重抓才不會留一堆幽靈列
        const offReconnect = onReconnect(() => { void refresh(); });

        return () => {
            unsubscribe("user_joined", onJoined);
            unsubscribe("user_left", onLeft);
            offReconnect();
        };
    }, [subscribe, unsubscribe, onReconnect, refresh]);

    const loggedInCount = rows.filter((r) => r.user_email).length;
    const visible = onlyLoggedIn ? rows.filter((r) => r.user_email) : rows;
    const selectedOnline = selectedAddr !== null && rows.some((r) => r.addr === selectedAddr);

    const selectTarget = (addr: string) => {
        setSelectedAddr(addr);
        setFocusToken((t) => t + 1);
    };

    return (
        <div className="flex flex-col gap-8">
            <section className="flex flex-col gap-3">
                <PageHeader
                    title={`WebSocket 連線（${rows.length}）`}
                    actions={
                        <>
                            <label className="flex items-center gap-2 text-sm text-neutral-600 dark:text-neutral-400 cursor-pointer select-none">
                                <input
                                    type="checkbox"
                                    checked={onlyLoggedIn}
                                    onChange={(e) => setOnlyLoggedIn(e.target.checked)}
                                    className="accent-primary-600"
                                />
                                只看已登入
                            </label>
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
                                className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 text-white text-sm font-medium rounded-sm transition-colors"
                            >
                                {refreshing
                                    ? <Loader2 className="w-4 h-4 animate-spin" />
                                    : <RefreshCw className="w-4 h-4" />}
                                重新整理
                            </button>
                        </>
                    }
                />

                <div className="text-xs text-neutral-500 dark:text-neutral-400">
                    其中 {loggedInCount} 個已登入
                    {" · "}
                    最後更新 {lastUpdated === null ? "—" : formatTimeOfDay(lastUpdated)}
                    {error && <span className="text-red-600 dark:text-red-400">{" · "}{error}</span>}
                </div>

                <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                    <div className="overflow-x-auto">
                        <AdminTable>
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh>真實 IP</AdminTh>
                                    <AdminTh className="hidden md:table-cell">連線位址</AdminTh>
                                    <AdminTh>使用者</AdminTh>
                                    <AdminTh className="hidden sm:table-cell">裝置</AdminTh>
                                    <AdminTh>連線時長</AdminTh>
                                    <AdminTh>操作</AdminTh>
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {visible.length === 0 ? (
                                    <AdminEmptyRow colSpan={6}>
                                        {onlyLoggedIn && rows.length > 0
                                            ? "目前沒有已登入的連線，取消勾選「只看已登入」可看全部。"
                                            : "目前沒有線上連線。前台訪客開啟頁面就會建立 WebSocket 連線，登入的管理員也會出現在這裡。"}
                                    </AdminEmptyRow>
                                ) : (
                                    visible.map((conn) => (
                                        <AdminRow
                                            key={conn.addr}
                                            className={selectedAddr === conn.addr ? "bg-primary-50 dark:bg-primary-950/40" : ""}
                                        >
                                            <AdminTd className="font-mono text-sm whitespace-nowrap">
                                                <span className="inline-flex items-center gap-1">
                                                    {conn.real_ip || "—"}
                                                    {conn.real_ip && <CopyButton value={conn.real_ip} label="真實 IP" />}
                                                </span>
                                            </AdminTd>
                                            <AdminTd className="hidden md:table-cell font-mono text-sm whitespace-nowrap">
                                                <span className="inline-flex items-center gap-1">
                                                    {conn.addr}
                                                    <CopyButton value={conn.addr} label="連線位址" />
                                                </span>
                                            </AdminTd>
                                            {/* wrap-break-word 而非 break-all：break-all 會讓這欄的
                                                min-content 掉到 1 字寬，鄰欄一擠就把 email 拆成一行一字 */}
                                            <AdminTd className="text-sm wrap-break-word">
                                                {conn.user_email ?? <span className="text-neutral-400">匿名訪客</span>}
                                            </AdminTd>
                                            <AdminTd className="hidden sm:table-cell text-sm">
                                                <span title={conn.user_agent || undefined}>
                                                    {deviceLabel(conn.user_agent)}
                                                </span>
                                            </AdminTd>
                                            <AdminTd className="text-sm whitespace-nowrap">
                                                <span title={formatDateTime(conn.connected_at)}>
                                                    {now === null
                                                        ? "—"
                                                        : formatDuration(now - new Date(conn.connected_at).getTime())}
                                                </span>
                                            </AdminTd>
                                            <AdminTd>
                                                <button
                                                    type="button"
                                                    onClick={() => selectTarget(conn.addr)}
                                                    className="flex items-center gap-1 px-2 py-1 whitespace-nowrap text-xs font-medium rounded-sm border border-primary-600 text-primary-700 dark:text-primary-300 hover:bg-primary-100 dark:hover:bg-primary-900 transition-colors"
                                                >
                                                    <Send className="w-3 h-3" />
                                                    發訊息
                                                </button>
                                            </AdminTd>
                                        </AdminRow>
                                    ))
                                )}
                            </tbody>
                        </AdminTable>
                    </div>
                </div>
            </section>

            <section>
                <h2 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100 mb-4">
                    發送訊息給指定連線
                </h2>
                <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg p-6">
                    {/* key 換目標就重掛，前一個目標的送出結果訊息不會殘留 */}
                    <SaySomethingForm
                        key={selectedAddr ?? "none"}
                        addr={selectedAddr}
                        online={selectedOnline}
                        focusToken={focusToken}
                    />
                </div>
            </section>
        </div>
    );
}
