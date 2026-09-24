"use client";

import type { Ref } from "react";
import { X } from "lucide-react";
import type { Log } from "@/types";
import { LEVEL_BADGE } from "@/libs/badge-styles";
import { formatDateTimeSeconds } from "@/libs/admin-datetime";
import { sortedFields } from "./log-groups";

/**
 * 單一 request_id 的完整軌跡（時間正序）。
 * 軌跡改用 drawer：塞在展開列裡會變成「表格→列→面板→軌跡」四層縮排，而且長軌跡會把表格撐爆。
 * 行為（Esc / 背景捲動鎖 / 焦點鎖）由呼叫端的 useDialog 提供，ref 從這裡掛上。
 */
export default function TraceDrawer({ trace, failed, onClose, dialogRef }: {
    trace: { requestId: string; rows: Log[] };
    /** true = 抓取失敗（與「這個 request_id 真的沒別的 log」文案要分得開） */
    failed: boolean;
    onClose: () => void;
    dialogRef: Ref<HTMLDivElement>;
}) {
    return (
        <div className="fixed inset-0 z-50 flex justify-end">
            <div className="absolute inset-0 bg-black/40" onClick={onClose} aria-hidden="true" />
            <div
                ref={dialogRef}
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
                        onClick={onClose}
                        aria-label="關閉"
                        className="shrink-0 p-1 rounded-sm text-neutral-500 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                    >
                        <X className="h-4 w-4" aria-hidden="true" />
                    </button>
                </div>

                {trace.rows.length === 0 ? (
                    <span className="text-sm text-neutral-500 dark:text-neutral-400">
                        {failed ? '軌跡載入失敗' : '查不到紀錄'}
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
    );
}
