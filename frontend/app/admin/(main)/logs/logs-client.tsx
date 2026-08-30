"use client";

import { Fragment, useState, useEffect, useCallback } from "react";
import { ChevronDown, ChevronRight, X } from "lucide-react";
import { getLogs, getLogTrace } from "@/api/logs";
import ErrorBanner, { LOAD_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import useFilterUrl from "@/hooks/useFilterUrl";
import usePolling from "@/hooks/usePolling";
import useDialog from "@/hooks/useDialog";
import type { Log, LogLevel } from "@/types";
import { LEVEL_BADGE, LEVEL_ROW_BG } from "@/libs/badge-styles";
import { formatDateTimeSeconds } from "@/libs/admin-datetime";
import { ADMIN_FILTER_INPUT } from "@/libs/input-styles";

const LIMIT = 100;
const COLUMNS = 6;
/** 自動刷新週期。usePolling 在背景分頁會跳過該次請求，所以不必怕擱著的分頁一直打後端 */
const REFRESH_MS = 15_000;
/** 連續重複的 log 合併成一列的時間窗（retry 這類事件常常一秒內連噴好幾筆） */
const DUPE_WINDOW_MS = 5 * 60 * 1000;
/** 沒有 fields / request_id，但訊息長到會被裁掉的也要能展開看全文 */
const LONG_MESSAGE = 120;

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

/**
 * fields 的顯示順序。`self` 一定排第一 —— 那是真正的錯誤原因
 * （message 只是 `System error occurred` 這種固定字串），其餘照請求上下文的閱讀順序。
 * 不在清單裡的 key 依字母序接在後面，所以新增 span field 不必改這裡。
 */
const FIELD_ORDER = ['self', 'panic', 'method', 'path', 'query', 'ip', 'status', 'latency_ms'];

function sortedFields(fields: Record<string, unknown>): [string, string][] {
    return Object.entries(fields)
        .map(([k, v]): [string, string] => [k, typeof v === 'string' ? v : JSON.stringify(v)])
        .sort(([a], [b]) => {
            const ia = FIELD_ORDER.indexOf(a);
            const ib = FIELD_ORDER.indexOf(b);
            if (ia !== -1 && ib !== -1) return ia - ib;
            if (ia !== -1) return -1;
            if (ib !== -1) return 1;
            return a.localeCompare(b);
        });
}

/** 一列 = 一個事件；連續重複的原始 log 收在 rows 裡（head 是最新那筆） */
interface LogGroup {
    head: Log;
    rows: Log[];
}

/**
 * 把**相鄰**且同層級／同來源／同訊息、時間相差在 DUPE_WINDOW_MS 內的 log 合併成一列。
 *
 * 只合併相鄰的（清單是新→舊），所以不會把中間夾著別的事件的兩筆黏在一起。
 * 合併掉的筆數不會消失 —— 列上標 `×N`，展開面板逐筆列出時間與 request_id；
 * 面板上方的 fields 一律取最新那筆。
 */
function groupConsecutive(logs: Log[]): LogGroup[] {
    const groups: LogGroup[] = [];
    for (const log of logs) {
        const last = groups[groups.length - 1];
        const prev = last?.rows[last.rows.length - 1];
        const sameEvent =
            last && last.head.level === log.level && last.head.target === log.target && last.head.message === log.message;
        const withinWindow =
            prev && Math.abs(new Date(prev.created_at).getTime() - new Date(log.created_at).getTime()) <= DUPE_WINDOW_MS;
        if (sameEvent && withinWindow) {
            last.rows.push(log);
        } else {
            groups.push({ head: log, rows: [log] });
        }
    }
    return groups;
}

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
                                groups.map(({ head: log, rows }) => {
                                    const fields = log.fields ?? {};
                                    const hasDetail =
                                        Object.keys(fields).length > 0 ||
                                        !!log.request_id ||
                                        rows.length > 1 ||
                                        log.message.length > LONG_MESSAGE;
                                    const isExpanded = expanded.has(log.id);
                                    const Chevron = isExpanded ? ChevronDown : ChevronRight;
                                    return (
                                        <Fragment key={log.id}>
                                            <AdminRow tone={LEVEL_ROW_BG[log.level]}>
                                                <AdminTd className="text-neutral-500 dark:text-neutral-500 font-mono align-top hidden sm:table-cell">{log.id}</AdminTd>
                                                <AdminTd className="align-top">
                                                    <span className={`px-2 py-0.5 rounded-sm text-xs font-semibold ${LEVEL_BADGE[log.level]}`}>
                                                        {log.level}
                                                    </span>
                                                </AdminTd>
                                                <AdminTd className="align-top">
                                                    {/* chevron 與訊息同一顆 button：整段可點、鍵盤可用，
                                                        焦點框吃全站那條 focus-visible 規則，不必自己補 */}
                                                    {hasDetail ? (
                                                        <button
                                                            onClick={() => toggleExpand(log.id)}
                                                            aria-expanded={isExpanded}
                                                            className="flex w-full items-start gap-1.5 text-left"
                                                        >
                                                            <Chevron className="mt-0.5 h-3.5 w-3.5 shrink-0 text-neutral-400" aria-hidden="true" />
                                                            {/* line-clamp 需要 display:-webkit-box，直接掛在 <td> 上會把
                                                                cell 從 table-cell 拔掉、整個表格排版壞掉，所以一定要有內層元素 */}
                                                            <span className={`grow min-w-0 font-mono wrap-break-word ${isExpanded ? '' : 'line-clamp-2'}`}>
                                                                {log.message}
                                                            </span>
                                                            {rows.length > 1 && (
                                                                <span
                                                                    title={rows.map(r => formatDateTimeSeconds(r.created_at)).join('\n')}
                                                                    className="shrink-0 px-1.5 py-0.5 rounded-sm text-xs font-medium bg-neutral-200 dark:bg-neutral-700 text-neutral-600 dark:text-neutral-300"
                                                                >
                                                                    ×{rows.length}
                                                                </span>
                                                            )}
                                                        </button>
                                                    ) : (
                                                        <div className="flex items-start gap-1.5">
                                                            <span className="w-3.5 shrink-0" aria-hidden="true" />
                                                            <span className="grow min-w-0 font-mono wrap-break-word line-clamp-2">{log.message}</span>
                                                        </div>
                                                    )}
                                                </AdminTd>
                                                <AdminTd
                                                    title={log.target}
                                                    className="text-neutral-600 dark:text-neutral-400 font-mono text-xs align-top truncate hidden lg:table-cell"
                                                >
                                                    {log.target}
                                                </AdminTd>
                                                <AdminTd
                                                    title={`${log.file}:${log.line}`}
                                                    className="text-neutral-600 dark:text-neutral-400 font-mono text-xs align-top truncate hidden xl:table-cell"
                                                >
                                                    {log.file}:{log.line}
                                                </AdminTd>
                                                <AdminTd className="text-neutral-500 dark:text-neutral-400 text-xs align-top whitespace-nowrap">
                                                    {formatDateTimeSeconds(log.created_at)}
                                                </AdminTd>
                                            </AdminRow>

                                            {isExpanded && (
                                                <tr>
                                                    <td
                                                        colSpan={COLUMNS}
                                                        className="border border-neutral-300 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800/40 px-4 py-3"
                                                    >
                                                        <div className="flex flex-col gap-3">
                                                            <div className="flex flex-col gap-1">
                                                                <span className="text-xs text-neutral-500 dark:text-neutral-400">完整訊息</span>
                                                                <pre className="font-mono text-xs whitespace-pre-wrap wrap-break-word text-neutral-800 dark:text-neutral-200">
                                                                    {log.message}
                                                                </pre>
                                                            </div>

                                                            {log.request_id && (
                                                                <div className="flex flex-wrap items-center gap-2">
                                                                    <span className="text-xs text-neutral-500 dark:text-neutral-400">request_id</span>
                                                                    <code className="font-mono text-xs break-all">{log.request_id}</code>
                                                                    <button
                                                                        onClick={() => showTrace(log.request_id!)}
                                                                        disabled={tracePendingId === log.request_id}
                                                                        className="px-2 py-0.5 rounded-sm text-xs font-medium bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 transition-colors"
                                                                    >
                                                                        {tracePendingId === log.request_id ? '載入中…' : '整條軌跡'}
                                                                    </button>
                                                                </div>
                                                            )}

                                                            {sortedFields(fields).map(([key, value]) => (
                                                                <div key={key} className="flex flex-col gap-1">
                                                                    <span className="text-xs text-neutral-500 dark:text-neutral-400">{key}</span>
                                                                    <pre className="font-mono text-xs whitespace-pre-wrap break-all text-neutral-800 dark:text-neutral-200">
                                                                        {value}
                                                                    </pre>
                                                                </div>
                                                            ))}

                                                            {rows.length > 1 && (
                                                                <div className="flex flex-col gap-1 border-t border-neutral-300 dark:border-neutral-700 pt-3">
                                                                    <span className="text-xs text-neutral-500 dark:text-neutral-400">
                                                                        合併的 {rows.length} 筆（上方 fields 取最新那筆）
                                                                    </span>
                                                                    {rows.map(row => (
                                                                        <div key={row.id} className="flex flex-wrap items-center gap-2 font-mono text-xs">
                                                                            <span className="text-neutral-500 dark:text-neutral-500">#{row.id}</span>
                                                                            <span className="text-neutral-500 dark:text-neutral-400">
                                                                                {formatDateTimeSeconds(row.created_at)}
                                                                            </span>
                                                                            {row.request_id && (
                                                                                <>
                                                                                    <code className="break-all">{row.request_id}</code>
                                                                                    <button
                                                                                        onClick={() => showTrace(row.request_id!)}
                                                                                        disabled={tracePendingId === row.request_id}
                                                                                        className="px-1.5 py-0.5 rounded-sm font-medium bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                                                                                    >
                                                                                        {tracePendingId === row.request_id ? '載入中…' : '軌跡'}
                                                                                    </button>
                                                                                </>
                                                                            )}
                                                                        </div>
                                                                    ))}
                                                                </div>
                                                            )}
                                                        </div>
                                                    </td>
                                                </tr>
                                            )}
                                        </Fragment>
                                    );
                                })
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

            {/* 軌跡改用 drawer：塞在展開列裡會變成「表格→列→面板→軌跡」四層縮排，
                而且長軌跡會把表格撐爆。行為（Esc / 背景捲動鎖 / 焦點鎖）全交給 useDialog */}
            {trace && (
                <div className="fixed inset-0 z-50 flex justify-end">
                    <div className="absolute inset-0 bg-black/40" onClick={closeTrace} aria-hidden="true" />
                    <div
                        ref={traceRef}
                        role="dialog"
                        aria-modal="true"
                        aria-label="請求的完整軌跡"
                        className="relative flex h-full w-full max-w-2xl flex-col gap-3 overflow-auto bg-white dark:bg-neutral-900 p-4 shadow-xl"
                    >
                        <div className="flex items-start justify-between gap-2">
                            <div className="flex flex-col gap-1 min-w-0">
                                <h2 className="text-sm font-semibold text-neutral-800 dark:text-neutral-100">
                                    請求的完整軌跡（時間正序，{trace.rows.length} 筆）
                                </h2>
                                <code className="font-mono text-xs break-all text-neutral-500 dark:text-neutral-400">
                                    {trace.requestId}
                                </code>
                            </div>
                            <button
                                onClick={closeTrace}
                                aria-label="關閉"
                                className="shrink-0 p-1 rounded-sm text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                            >
                                <X className="h-4 w-4" aria-hidden="true" />
                            </button>
                        </div>

                        {trace.rows.length === 0 ? (
                            <span className="text-sm text-neutral-500 dark:text-neutral-400">
                                {traceFailed ? '軌跡載入失敗' : '查不到紀錄'}
                            </span>
                        ) : (
                            <div className="flex flex-col gap-2">
                                {trace.rows.map(row => (
                                    <div key={row.id} className="flex flex-col gap-1 border-b border-neutral-200 dark:border-neutral-800 pb-2 last:border-b-0">
                                        <div className="flex flex-wrap items-center gap-2 font-mono text-xs">
                                            <span className="text-neutral-500 dark:text-neutral-400">
                                                {formatDateTimeSeconds(row.created_at)}
                                            </span>
                                            <span className={`px-1.5 rounded-sm ${LEVEL_BADGE[row.level]}`}>{row.level}</span>
                                            <span className="text-neutral-600 dark:text-neutral-400 break-all">{row.target}</span>
                                        </div>
                                        <pre className="font-mono text-xs whitespace-pre-wrap wrap-break-word text-neutral-800 dark:text-neutral-200">
                                            {row.message}
                                        </pre>
                                        {sortedFields(row.fields ?? {}).map(([key, value]) => (
                                            <div key={key} className="flex flex-wrap gap-2 font-mono text-xs">
                                                <span className="shrink-0 text-neutral-500 dark:text-neutral-400">{key}</span>
                                                <span className="break-all text-neutral-700 dark:text-neutral-300">{value}</span>
                                            </div>
                                        ))}
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}
