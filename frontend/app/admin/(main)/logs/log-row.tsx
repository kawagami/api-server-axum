"use client";

import { ChevronDown, ChevronRight } from "lucide-react";
import { AdminRow, AdminTd } from "@/components/admin/table";
import { LEVEL_BADGE, LEVEL_ROW_BG } from "@/libs/badge-styles";
import { formatDateTimeSeconds } from "@/libs/admin-datetime";
import { sortedFields, type LogGroup } from "./log-groups";

/** 表格欄數（展開列的 colSpan 與空狀態列共用） */
export const COLUMNS = 6;
/** 沒有 fields / request_id，但訊息長到會被裁掉的也要能展開看全文 */
const LONG_MESSAGE = 120;

/** 一個事件一列；可展開看完整訊息、fields、request_id（含「整條軌跡」）與合併掉的重複筆 */
export default function LogRow({ group, expanded: isExpanded, onToggle, onTrace, tracePendingId }: {
    group: LogGroup;
    expanded: boolean;
    onToggle: (id: number) => void;
    onTrace: (requestId: string) => void;
    tracePendingId: string | null;
}) {
    const { head: log, rows } = group;
    const fields = log.fields ?? {};
    const hasDetail =
        Object.keys(fields).length > 0 ||
        !!log.request_id ||
        rows.length > 1 ||
        log.message.length > LONG_MESSAGE;
    const Chevron = isExpanded ? ChevronDown : ChevronRight;
    return (
        <>
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
                            onClick={() => onToggle(log.id)}
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
                                        onClick={() => onTrace(log.request_id!)}
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
                                                        onClick={() => onTrace(row.request_id!)}
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
        </>
    );
}
