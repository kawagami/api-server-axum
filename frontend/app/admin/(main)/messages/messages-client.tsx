"use client";

import { useEffect, useState } from "react";
import { Trash2, Mail, ChevronRight, ChevronDown } from "lucide-react";
import { getContactMessages, deleteContactMessage } from "@/api/contact";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import AdminTableContainer from "@/components/admin/admin-table-container";
import ErrorBanner, { LOAD_FAILED, DELETE_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import usePagedList from "@/hooks/usePagedList";
import { formatDateTime } from "@/libs/admin-datetime";
import type { ContactMessage } from "@/types";

const LIMIT = 50;
/** 收合時只給 3 行；比這短的內容本來就顯示得完，不必長出展開鈕 */
const CLAMP_LINES = 3;
const LONG_CONTENT = 120;

/** 內容夠短就整段顯示，不必給展開鈕（垃圾訊息動輒數十行，短留言一兩行） */
function isLong(content: string) {
    return content.length > LONG_CONTENT || content.split('\n').length > CLAMP_LINES;
}

export default function MessagesClient({ canDelete }: { canDelete: boolean }) {
    const { items: messages, total, hasMore, isPending, failed, load, loadMore, setItems } =
        usePagedList<ContactMessage>();
    const [deletingId, setDeletingId] = useState<number | null>(null);
    const [deleteError, setDeleteError] = useState<string | null>(null);
    // 可同時展開多列（要比對兩筆時不必來回點）
    const [expanded, setExpanded] = useState<Set<number>>(() => new Set());

    useEffect(() => {
        load(page => getContactMessages(page, LIMIT));
    }, [load]);

    function toggleExpand(id: number) {
        setExpanded(prev => {
            const next = new Set(prev);
            if (!next.delete(id)) next.add(id);
            return next;
        });
    }

    async function handleDelete(id: number) {
        if (deletingId) return;
        if (!window.confirm("確定要刪除這則留言嗎？")) return;
        setDeleteError(null);
        setDeletingId(id);
        try {
            await deleteContactMessage(id);
            setItems(prev => prev.filter(m => m.id !== id));
        } catch {
            setDeleteError(DELETE_FAILED);
        } finally {
            setDeletingId(null);
        }
    }

    const colSpan = canDelete ? 4 : 3;

    return (
        // 高度鏈：layout 的 h-full flex 欄 → 這裡 flex-1 → 表格區 flex-1 → AdminTableContainer fill
        <div className="flex min-h-0 flex-1 flex-col gap-4">
            <PageHeader
                title="訪客留言"
                description={`訪客從前台聯絡表單送來的訊息，共 ${total} 筆，已載入 ${messages.length} 筆`}
            />

            <ErrorBanner message={failed ? LOAD_FAILED : deleteError} />

            <div className={`flex min-h-0 flex-1 flex-col transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                <AdminTableContainer stickyHead fill>
                    {/* table-fixed：auto layout 下一則垃圾訊息就會把內容欄撐成整個視窗高，
                        時間／來自欄反而被壓到最窄。固定配寬讓內容吃剩下的寬度，行數由 line-clamp 控。 */}
                    <AdminTable className="text-sm table-fixed">
                        <thead>
                            <AdminHeadRow>
                                <AdminTh className="col-datetime">時間</AdminTh>
                                <AdminTh className="w-[15em]">來自</AdminTh>
                                <AdminTh>內容</AdminTh>
                                {canDelete && <AdminTh className="col-badge"><span className="sr-only">操作</span></AdminTh>}
                            </AdminHeadRow>
                        </thead>
                        <tbody>
                            {messages.length === 0 ? (
                                <AdminEmptyRow colSpan={colSpan}>
                                    {isPending ? '載入中…' : '目前沒有留言'}
                                </AdminEmptyRow>
                            ) : (
                                messages.map(m => {
                                    const long = isLong(m.content);
                                    const isExpanded = expanded.has(m.id);
                                    const Chevron = isExpanded ? ChevronDown : ChevronRight;
                                    return (
                                        <AdminRow key={m.id}>
                                            <AdminTd className="whitespace-nowrap align-top text-xs text-neutral-500 dark:text-neutral-400">
                                                {formatDateTime(m.created_at)}
                                            </AdminTd>
                                            <AdminTd className="align-top text-xs wrap-break-word">
                                                <div className="text-neutral-800 dark:text-neutral-200">{m.name || '匿名'}</div>
                                                {/* 不用 inline-flex：長 email 折行時那種盒子只有第一行參與行框，
                                                    其餘行會溢出疊到下面的元素（gov_tenders 標題就踩過） */}
                                                {m.email && (
                                                    <a href={`mailto:${m.email}`} className="text-primary-700 dark:text-primary-300 hover:underline">
                                                        <Mail className="mr-1 inline-block h-3 w-3 align-[-1px]" />
                                                        {m.email}
                                                    </a>
                                                )}
                                            </AdminTd>
                                            <AdminTd className="align-top text-neutral-800 dark:text-neutral-200">
                                                {long ? (
                                                    <button
                                                        onClick={() => toggleExpand(m.id)}
                                                        aria-expanded={isExpanded}
                                                        title={isExpanded ? '收合' : '展開全文'}
                                                        className="flex w-full items-start gap-1.5 text-left"
                                                    >
                                                        <Chevron className="mt-0.5 h-3.5 w-3.5 shrink-0 text-neutral-400" aria-hidden="true" />
                                                        {/* line-clamp 需要 display:-webkit-box，直接掛在 <td> 上會把
                                                            cell 從 table-cell 拔掉、整個表格排版壞掉，所以一定要有內層元素 */}
                                                        <span className={`grow min-w-0 whitespace-pre-wrap wrap-break-word ${isExpanded ? '' : 'line-clamp-3'}`}>
                                                            {m.content}
                                                        </span>
                                                    </button>
                                                ) : (
                                                    <div className="flex items-start gap-1.5">
                                                        <span className="w-3.5 shrink-0" aria-hidden="true" />
                                                        <span className="grow min-w-0 whitespace-pre-wrap wrap-break-word">{m.content}</span>
                                                    </div>
                                                )}
                                            </AdminTd>
                                            {canDelete && (
                                                <AdminTd className="whitespace-nowrap align-top">
                                                    <button
                                                        onClick={() => handleDelete(m.id)}
                                                        disabled={deletingId === m.id}
                                                        className="inline-flex items-center gap-1 px-2 py-1 text-xs rounded-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40 disabled:opacity-50 transition-colors"
                                                    >
                                                        <Trash2 className="w-3.5 h-3.5" />
                                                        刪除
                                                    </button>
                                                </AdminTd>
                                            )}
                                        </AdminRow>
                                    );
                                })
                            )}
                        </tbody>
                    </AdminTable>
                </AdminTableContainer>
            </div>

            {hasMore && (
                <div className="flex shrink-0 justify-center">
                    <button
                        onClick={loadMore}
                        disabled={isPending}
                        className="px-6 py-2 bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 rounded-sm hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 text-sm font-medium transition-colors"
                    >
                        {isPending ? '載入中…' : '載入更多'}
                    </button>
                </div>
            )}
        </div>
    );
}
