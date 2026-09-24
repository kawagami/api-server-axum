"use client";

import { useState, useEffect, useCallback } from "react";
import { getLogTrace } from "@/api/logs";
import { getLogs } from "@/libs/admin-queries";
import ErrorBanner, { LOAD_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminTh, AdminEmptyRow } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import useFilterUrl from "@/hooks/useFilterUrl";
import usePolling from "@/hooks/usePolling";
import useDialog from "@/hooks/useDialog";
import type { Log, LogLevel } from "@/types";
import { ADMIN_FILTER_INPUT } from "@/libs/input-styles";
import { groupConsecutive } from "./log-groups";
import LogRow, { COLUMNS } from "./log-row";
import TraceDrawer from "./trace-drawer";

const LIMIT = 100;
/** 自動刷新週期。usePolling 在背景分頁會跳過該次請求，所以不必怕擱著的分頁一直打後端 */
const REFRESH_MS = 15_000;

type LevelFilter = '' | LogLevel;

const LEVEL_FILTERS: { value: LevelFilter; label: string }[] = [
    { value: '', label: '全部' },
    { value: 'INFO', label: 'INFO' },
    { value: 'WARN', label: 'WARN' },
    { value: 'ERROR', label: 'ERROR' },
];

const VALID_LEVELS: LevelFilter[] = LEVEL_FILTERS.map(f => f.value);
const defaultFilters = { level: '', q: '' };

const CHIP = "px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-50";
const CHIP_ON = "bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900";
const CHIP_OFF = "bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200 dark:hover:bg-neutral-700";

export default function LogsClient() {
    const { items: logs, total, hasMore, isPending, failed, load, loadMore } = usePagedList<Log>();
    const { initial, write } = useFilterUrl(defaultFilters);
    // URL 是使用者可以亂打的，不在白名單內的 level 一律當成「全部」
    const [level, setLevel] = useState<LevelFilter>(
        () => (VALID_LEVELS.includes(initial.level as LevelFilter) ? initial.level as LevelFilter : '')
    );
    const [q, setQ] = useState(initial.q ?? '');
    const [appliedQ, setAppliedQ] = useState(initial.q ?? '');
    // 可同時展開多列（要比對兩筆時不必來回點）
    const [expanded, setExpanded] = useState<Set<number>>(() => new Set());
    const [autoRefresh, setAutoRefresh] = useState(false);
    // 同一個 request_id 的完整軌跡（時間正序）。只快取最近查的那一筆就夠用
    const [trace, setTrace] = useState<{ requestId: string; rows: Log[] } | null>(null);
    // 記的是 request_id 而非布林 —— 布林會讓所有列的軌跡鈕一起 disabled
    const [tracePendingId, setTracePendingId] = useState<string | null>(null);
    const [traceFailed, setTraceFailed] = useState(false);

    useEffect(() => {
        load(page => getLogs({ level: level || undefined, q: appliedQ || undefined, page, per_page: LIMIT }));
        // 初次載入沿用 URL 帶進來的條件；後續改條件走 handleFilterChange / handleSearch
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [load]);

    function reload(nextLevel: LevelFilter, nextQ: string) {
        write({ level: nextLevel, q: nextQ });
        load(page => getLogs({ level: nextLevel || undefined, q: nextQ || undefined, page, per_page: LIMIT }));
    }

    /** 用當前條件重抓第 1 頁（自動刷新用；條件本身沒變，所以不寫 URL） */
    function refresh() {
        load(page => getLogs({ level: level || undefined, q: appliedQ || undefined, page, per_page: LIMIT }));
    }

    // 輪詢只重抓第 1 頁，所以開啟期間「載入更多」會停用（見 toggleAutoRefresh）
    usePolling(() => {
        if (!isPending) refresh();
    }, REFRESH_MS, autoRefresh);

    function toggleAutoRefresh() {
        const next = !autoRefresh;
        setAutoRefresh(next);
        // 開啟時立刻收回第 1 頁：不然已按過「載入更多」的內容會在第一次輪詢時莫名消失
        if (next) refresh();
    }

    function handleFilterChange(newLevel: LevelFilter) {
        if (newLevel === level || isPending) return;
        setLevel(newLevel);
        reload(newLevel, appliedQ);
    }

    function handleSearch() {
        if (isPending) return;
        setAppliedQ(q);
        reload(level, q);
    }

    function handleClear() {
        if (isPending) return;
        setQ('');
        setAppliedQ('');
        reload(level, '');
    }

    function handleLoadMore() {
        if (isPending || autoRefresh) return;
        loadMore();
    }

    function toggleExpand(id: number) {
        setExpanded(prev => {
            const next = new Set(prev);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            return next;
        });
    }

    const closeTrace = useCallback(() => setTrace(null), []);
    const traceRef = useDialog<HTMLDivElement>(trace !== null, closeTrace);

    async function showTrace(requestId: string) {
        if (tracePendingId) return;
        setTracePendingId(requestId);
        setTraceFailed(false);
        try {
            setTrace({ requestId, rows: await getLogTrace(requestId) });
        } catch {
            // 抓不到與「這個 request_id 真的沒別的 log」是兩件事，文案要分得開
            setTrace({ requestId, rows: [] });
            setTraceFailed(true);
        } finally {
            setTracePendingId(null);
        }
    }

    const groups = groupConsecutive(logs);

    return (
        // 高度鏈：layout 的 h-full flex 欄 → 這裡 flex-1 → 表格區 flex-1 → AdminTableContainer fill。
        // 任一層漏掉 min-h-0 就會被內容撐開，外層的 overflow-auto 又會長出第二條捲軸。
        <div className="flex min-h-0 flex-1 flex-col gap-4">
            {/* 統計併進 description 而不是自己佔一列：整頁剛好塞滿一屏，多一列就會把表格擠掉一列高度 */}
            <PageHeader
                title="系統日誌"
                description={`共 ${total} 筆，已載入 ${logs.length} 筆${
                    groups.length !== logs.length ? `（合併重複後 ${groups.length} 列）` : ''
                }`}
                actions={
                    <button
                        onClick={toggleAutoRefresh}
                        aria-pressed={autoRefresh}
                        title={`每 ${REFRESH_MS / 1000} 秒重抓第 1 頁`}
                        className={`${CHIP} ${autoRefresh ? CHIP_ON : CHIP_OFF}`}
                    >
                        自動刷新{autoRefresh ? '中' : ''}
                    </button>
                }
            />

            {/* 篩選一律收在這張灰底卡片裡（與 audit_logs / gov_tenders 同一套版型），
                PageHeader 的動作區只放「自動刷新」這種與查詢條件無關的開關 */}
            <div className="flex flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                <div className="flex flex-col gap-1">
                    <span className="text-xs text-neutral-500 dark:text-neutral-400">層級</span>
                    <div className="flex flex-wrap gap-1">
                        {LEVEL_FILTERS.map(({ value, label }) => (
                            <button
                                key={value || 'ALL'}
                                onClick={() => handleFilterChange(value)}
                                disabled={isPending}
                                aria-pressed={level === value}
                                className={`${CHIP} ${level === value ? CHIP_ON : CHIP_OFF}`}
                            >
                                {label}
                            </button>
                        ))}
                    </div>
                </div>

                {/* 搜尋 message 與 fields —— 錯誤細節在 fields.self，只搜 message 找不到有用的東西 */}
                <div className="flex flex-col gap-1 grow min-w-60">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">
                        關鍵字（同時比對訊息與錯誤細節）
                    </label>
                    <input
                        type="text"
                        value={q}
                        onChange={e => setQ(e.target.value)}
                        onKeyDown={e => e.key === 'Enter' && handleSearch()}
                        placeholder="例：Cannot assign requested address"
                        className={`${ADMIN_FILTER_INPUT} w-full`}
                    />
                </div>
                <button
                    onClick={handleSearch}
                    disabled={isPending}
                    className="px-4 py-1.5 text-sm font-medium rounded-sm bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50 transition-colors"
                >
                    搜尋
                </button>
                {(q || appliedQ) && (
                    <button
                        onClick={handleClear}
                        disabled={isPending}
                        className={`${CHIP} ${CHIP_OFF}`}
                    >
                        清除
                    </button>
                )}
            </div>

            <ErrorBanner message={failed ? LOAD_FAILED : null} />

            <div className={`flex min-h-0 flex-1 flex-col transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                <AdminTableContainer stickyHead fill>
                    {/* table-fixed：全後台只有這張表用。auto layout 下訊息欄的 min-content
                        會被 break 掉的字元拉到 1 字寬，而來源模組／檔案的長 token 不可斷、
                        反過來把寬度全吃走 —— 最該讀的欄位變最窄。固定配寬讓訊息吃剩下全部。 */}
                    <AdminTable className="text-sm table-fixed">
                        <thead>
                            <AdminHeadRow>
                                <AdminTh className="col-id hidden sm:table-cell">ID</AdminTh>
                                <AdminTh className="col-badge">層級</AdminTh>
                                <AdminTh>訊息</AdminTh>
                                <AdminTh className="w-[14em] hidden lg:table-cell">來源模組</AdminTh>
                                <AdminTh className="w-[17em] hidden xl:table-cell">檔案</AdminTh>
                                <AdminTh className="col-datetime">時間</AdminTh>
                            </AdminHeadRow>
                        </thead>
                        <tbody>
                            {groups.length === 0 ? (
                                <AdminEmptyRow colSpan={COLUMNS}>
                                    {isPending ? '載入中…' : '目前沒有日誌'}
                                </AdminEmptyRow>
                            ) : (
                                groups.map(group => (
                                    <LogRow
                                        key={group.head.id}
                                        group={group}
                                        expanded={expanded.has(group.head.id)}
                                        onToggle={toggleExpand}
                                        onTrace={showTrace}
                                        tracePendingId={tracePendingId}
                                    />
                                ))
                            )}
                        </tbody>
                    </AdminTable>
                </AdminTableContainer>
            </div>

            {hasMore && (
                <div className="flex shrink-0 flex-col items-center gap-1">
                    <button
                        onClick={handleLoadMore}
                        disabled={isPending || autoRefresh}
                        className="px-6 py-2 bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 rounded-sm hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 text-sm font-medium transition-colors"
                    >
                        {isPending ? '載入中…' : '載入更多'}
                    </button>
                    {autoRefresh && (
                        <span className="text-xs text-neutral-500 dark:text-neutral-400">
                            自動刷新會重抓第 1 頁，關閉後才能往下載入
                        </span>
                    )}
                </div>
            )}

            {trace && (
                <TraceDrawer trace={trace} failed={traceFailed} onClose={closeTrace} dialogRef={traceRef} />
            )}
        </div>
    );
}
