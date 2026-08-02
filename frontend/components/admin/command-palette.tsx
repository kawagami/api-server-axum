"use client";

import { useEffect, useId, useMemo, useRef, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { Search } from "lucide-react";
import useDialog from "@/hooks/useDialog";
import type { AdminNavGroup } from "@/components/admin/nav";

interface Entry {
    label: string;
    href: string;
    group: string;
    icon: AdminNavGroup["icon"];
}

/** 攤平成單層清單；groups 已由呼叫端依權限與功能開關過濾過，這裡不再做授權判斷 */
function flatten(groups: AdminNavGroup[]): Entry[] {
    return groups.flatMap(group =>
        group.items.map(item => ({
            label: item.label,
            href: item.href,
            group: group.label,
            icon: group.icon,
        })),
    );
}

/** 標籤、分組名、路徑任一命中即算符合（路徑可比對，方便記得英文 route 的人直接打） */
function matches(entry: Entry, query: string): boolean {
    const q = query.trim().toLowerCase();
    if (!q) return true;
    return (
        entry.label.toLowerCase().includes(q) ||
        entry.group.toLowerCase().includes(q) ||
        entry.href.toLowerCase().includes(q)
    );
}

/**
 * 後台快速跳頁（⌘K / Ctrl+K）。
 * 選單來源與側邊欄共用 `adminNavGroups`，所以新增頁面不必再動這個元件。
 */
export default function AdminCommandPalette({
    groups,
    onClose,
}: {
    groups: AdminNavGroup[];
    onClose: () => void;
}) {
    const router = useRouter();
    const listId = useId();
    const [query, setQuery] = useState("");
    const [activeIndex, setActiveIndex] = useState(0);
    const listRef = useRef<HTMLUListElement>(null);
    // Esc 關閉、鎖背景捲動、焦點鎖在對話框內（開啟時自動聚焦到搜尋框）
    const dialogRef = useDialog<HTMLDivElement>(true, onClose);

    const entries = useMemo(() => flatten(groups), [groups]);
    const results = useMemo(() => entries.filter(e => matches(e, query)), [entries, query]);

    // 結果變短時把選取位置收回範圍內
    const active = results.length > 0 ? Math.min(activeIndex, results.length - 1) : -1;

    useEffect(() => {
        listRef.current
            ?.querySelector<HTMLElement>('[data-active="true"]')
            ?.scrollIntoView({ block: "nearest" });
    }, [active]);

    function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
        if (e.key === "ArrowDown") {
            e.preventDefault();
            setActiveIndex(i => (results.length === 0 ? 0 : (Math.min(i, results.length - 1) + 1) % results.length));
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            setActiveIndex(i => (results.length === 0 ? 0 : (Math.min(i, results.length - 1) + results.length - 1) % results.length));
        } else if (e.key === "Enter") {
            e.preventDefault();
            const target = results[active];
            if (target) {
                router.push(target.href);
                onClose();
            }
        }
    }

    return (
        <div
            className="fixed inset-0 z-50 bg-black/40 flex items-start justify-center p-4 pt-[10vh]"
            onClick={onClose}
        >
            <div
                ref={dialogRef}
                role="dialog"
                aria-modal="true"
                aria-label="快速跳頁"
                className="w-full max-w-lg bg-white dark:bg-neutral-900 rounded-lg shadow-xl overflow-hidden"
                onClick={e => e.stopPropagation()}
            >
                <div className="flex items-center gap-2 px-4 py-3 border-b border-neutral-200 dark:border-neutral-700">
                    <Search size={16} className="shrink-0 text-neutral-400" />
                    <input
                        type="text"
                        role="combobox"
                        aria-expanded
                        aria-controls={listId}
                        aria-activedescendant={active >= 0 ? `${listId}-${active}` : undefined}
                        autoComplete="off"
                        value={query}
                        onChange={e => {
                            setQuery(e.target.value);
                            setActiveIndex(0);
                        }}
                        onKeyDown={handleKeyDown}
                        placeholder="搜尋後台頁面…"
                        className="flex-1 min-w-0 bg-transparent text-sm text-neutral-900 dark:text-neutral-100 placeholder-neutral-400 border-0 p-0"
                    />
                    <kbd className="shrink-0 text-[10px] text-neutral-400 border border-neutral-200 dark:border-neutral-700 rounded-sm px-1.5 py-0.5">
                        Esc
                    </kbd>
                </div>

                <ul
                    ref={listRef}
                    id={listId}
                    role="listbox"
                    aria-label="頁面"
                    className="max-h-[50vh] overflow-y-auto py-1"
                >
                    {results.length === 0 ? (
                        <li className="px-4 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400">
                            找不到符合的頁面
                        </li>
                    ) : (
                        results.map((entry, i) => {
                            const Icon = entry.icon;
                            const isActive = i === active;
                            return (
                                <li key={entry.href} role="none">
                                    <Link
                                        id={`${listId}-${i}`}
                                        role="option"
                                        aria-selected={isActive}
                                        data-active={isActive}
                                        href={entry.href}
                                        onClick={onClose}
                                        onMouseEnter={() => setActiveIndex(i)}
                                        // 滑鼠移入即同步選取位置，鍵盤與滑鼠不會各指一個
                                        tabIndex={-1}
                                        className={`flex items-center gap-3 px-4 py-2 text-sm transition-colors ${isActive
                                            ? "bg-primary-50 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300"
                                            : "text-neutral-700 dark:text-neutral-200 hover:bg-neutral-100 dark:hover:bg-neutral-800"
                                            }`}
                                    >
                                        <Icon size={16} className="shrink-0 text-neutral-400" />
                                        <span className="flex-1 min-w-0 truncate">{entry.label}</span>
                                        <span className="shrink-0 text-xs text-neutral-400">{entry.group}</span>
                                    </Link>
                                </li>
                            );
                        })
                    )}
                </ul>
            </div>
        </div>
    );
}
