"use client";

import { useEffect, useRef } from "react";
import { X } from "lucide-react";
import type { Selection } from "@/hooks/useSelection";

/**
 * 後台清單多選的共用 UI，狀態由 `hooks/useSelection.ts` 提供。
 *
 * 用法（範本 `app/admin/(main)/messages/messages-client.tsx`）：
 * - 表頭 `<AdminTh className="col-check"><SelectAllCheckbox selection={…} /></AdminTh>`
 * - 每列 `<AdminTd className="col-check"><RowCheckbox selection={…} id={row.id} label="選取…" /></AdminTd>`
 * - 有勾選時在 `PageHeader` 的 `actions` 放 `<BulkActionBar>`：放在標題列而不是表格上方，
 *   勾第一筆時表格才不會被往下推（正要點第二筆時列跑掉）
 *
 * 單次批次上限見後端 `structs/batch.rs` 的 `MAX_BATCH_IDS`（ids 放 URL query，有長度上限）。
 */
export const MAX_BATCH_IDS = 100;

/** 勾選框外包一層撐滿格子的 label：整格都可點，不必瞄準 13px 的框 */
function CheckboxCell({
    checked,
    indeterminate = false,
    label,
    onToggle,
}: {
    checked: boolean;
    indeterminate?: boolean;
    label: string;
    onToggle: (shiftKey: boolean) => void;
}) {
    const ref = useRef<HTMLInputElement>(null);
    // indeterminate 只能用 DOM property 設，沒有對應的 HTML attribute
    useEffect(() => {
        if (ref.current) ref.current.indeterminate = indeterminate;
    }, [indeterminate]);

    return (
        <label className="-mx-4 -my-2 flex cursor-pointer justify-center px-4 py-2">
            <input
                ref={ref}
                type="checkbox"
                checked={checked}
                aria-label={label}
                // shift + 點擊時瀏覽器會把兩次點擊之間的文字整片反白，擋掉 mousedown 的預設行為
                onMouseDown={e => { if (e.shiftKey) e.preventDefault(); }}
                // checkbox 的 onChange 由 click 觸發，nativeEvent 是滑鼠事件；鍵盤空白鍵則沒有 shiftKey
                onChange={e => onToggle("shiftKey" in e.nativeEvent && e.nativeEvent.shiftKey === true)}
                className="accent-primary-600"
            />
        </label>
    );
}

export function SelectAllCheckbox<K extends number | string>({ selection }: { selection: Selection<K> }) {
    return (
        <CheckboxCell
            checked={selection.allSelected}
            indeterminate={selection.someSelected}
            label="全選已載入的列"
            onToggle={() => selection.toggleAll()}
        />
    );
}

export function RowCheckbox<K extends number | string>({
    selection,
    id,
    label,
}: {
    selection: Selection<K>;
    id: K;
    label: string;
}) {
    return (
        <CheckboxCell
            checked={selection.isSelected(id)}
            label={label}
            onToggle={shiftKey => selection.toggle(id, shiftKey)}
        />
    );
}

/**
 * 「已選 N 筆」+ 批次動作按鈕（children）+ 取消選取。沒有勾選時不渲染。
 * 超過 `MAX_BATCH_IDS` 時顯示提示，呼叫端的動作按鈕應以 `selection.count > MAX_BATCH_IDS` 停用。
 */
export function BulkActionBar<K extends number | string>({
    selection,
    children,
}: {
    selection: Selection<K>;
    children: React.ReactNode;
}) {
    if (selection.count === 0) return null;
    const over = selection.count > MAX_BATCH_IDS;
    return (
        <div className="flex flex-wrap items-center gap-2 text-sm" role="status">
            <span className={over ? "text-red-600 dark:text-red-400" : "text-neutral-600 dark:text-neutral-300"}>
                已選 {selection.count} 筆{over && `（一次最多 ${MAX_BATCH_IDS} 筆）`}
            </span>
            {children}
            <button
                onClick={selection.clear}
                className="inline-flex items-center gap-1 rounded-sm px-2 py-1 text-xs text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
            >
                <X className="h-3.5 w-3.5" />
                取消選取
            </button>
        </div>
    );
}
