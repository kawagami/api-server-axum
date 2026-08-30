"use client";

import { useEffect, useRef, useState } from "react";
import { Loader2 } from "lucide-react";
import Modal from "@/components/modal";
import type { TagCount } from "@/types";

// 與後端 normalize_tags 同規則：trim + 不分大小寫視為同一個 tag
const tagKey = (tag: string) => tag.trim().toLowerCase();

/**
 * tag 改名 / 合併。取代原本的 `window.prompt` —— prompt 除了樣式不受控，
 * 還無法在送出前告訴使用者「這個目標 tag 已經存在，按下去是合併不是改名」，
 * 而合併是不可逆的（合併後兩個 tag 的文章再也分不開）。
 */
export default function TagRenameModal({
    tag,
    tags,
    busy,
    onConfirm,
    onClose,
}: {
    tag: string;
    tags: TagCount[];
    busy: boolean;
    onConfirm: (to: string) => void;
    onClose: () => void;
}) {
    const [value, setValue] = useState(tag);
    const inputRef = useRef<HTMLInputElement>(null);
    // Modal 內的 useDialog 會聚焦第一個可聚焦元素，但這個對話框是拿來打字的
    // —— Modal 是子元件、effect 先跑，這支後跑，所以覆寫得掉
    useEffect(() => { inputRef.current?.select(); }, []);

    const trimmed = value.trim();
    const unchanged = !trimmed || tagKey(trimmed) === tagKey(tag);
    // 目標已存在 = 這是合併，不是改名
    const mergeTarget = tags.find(t => tagKey(t.tag) === tagKey(trimmed) && tagKey(t.tag) !== tagKey(tag));

    return (
        <Modal label={`改名 ${tag}`} onClose={onClose} dismissible={!busy} className="p-6">
            <h2 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100 mb-1">
                改名或合併 tag
            </h2>
            <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-4">
                來源「{tag}」，只影響你有權限的文章。
            </p>

            <label htmlFor="tag-rename-to" className="block text-xs text-neutral-500 dark:text-neutral-400 mb-1">
                改為
            </label>
            <input
                id="tag-rename-to"
                ref={inputRef}
                list="tag-rename-candidates"
                value={value}
                onChange={e => setValue(e.target.value)}
                onKeyDown={e => {
                    if (e.key === 'Enter' && !e.nativeEvent.isComposing && !unchanged && !busy) onConfirm(trimmed);
                }}
                className="w-full p-2 border rounded-sm border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100"
                placeholder="新名稱，或既有 tag（＝合併）"
            />
            <datalist id="tag-rename-candidates">
                {tags.filter(t => tagKey(t.tag) !== tagKey(tag)).map(t => (
                    <option key={t.tag} value={t.tag} />
                ))}
            </datalist>

            <p className="mt-2 min-h-8 text-xs text-neutral-500 dark:text-neutral-400">
                {mergeTarget
                    ? `「${mergeTarget.tag}」已存在（${mergeTarget.count} 篇），這會把兩者合併，合併後無法分開。`
                    : 'tag 不分大小寫；輸入既有 tag 即為合併。'}
            </p>

            <div className="flex gap-2 mt-4">
                <button
                    onClick={onClose}
                    disabled={busy}
                    className="flex-1 px-4 py-2 rounded-lg bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                >
                    取消
                </button>
                <button
                    onClick={() => onConfirm(trimmed)}
                    disabled={busy || unchanged}
                    className="flex-1 flex items-center justify-center gap-1 px-4 py-2 rounded-lg bg-primary-600 hover:bg-primary-700 text-white disabled:bg-neutral-500 disabled:cursor-not-allowed transition-colors"
                >
                    {busy && <Loader2 className="w-4 h-4 animate-spin" />}
                    {mergeTarget ? '合併' : '改名'}
                </button>
            </div>
        </Modal>
    );
}
