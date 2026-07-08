"use client";

import { useState, useEffect, useMemo, useRef } from 'react';
import { Check, X } from 'lucide-react';

interface Props {
    tags: string[];
    allTags: string[];
    onTagsChange: (tags: string[]) => void;
    onClose: () => void;
}

// 與後端 normalize_tags 同規則:trim + 不分大小寫視為同一個 tag
const tagKey = (tag: string) => tag.trim().toLowerCase();

export default function TagEditorModal({ tags, allTags, onTagsChange, onClose }: Props) {
    const [input, setInput] = useState('');
    const [hint, setHint] = useState<string | null>(null);
    const inputRef = useRef<HTMLInputElement>(null);
    const dialogRef = useRef<HTMLDivElement>(null);

    // Close the tag modal on Escape.
    useEffect(() => {
        const handler = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
        window.addEventListener('keydown', handler);
        return () => window.removeEventListener('keydown', handler);
    }, [onClose]);

    useEffect(() => { inputRef.current?.focus(); }, []);

    // 候選 = 全站 tags ∪ 本文 tags(含本次新建、尚未存檔的)
    const candidates = useMemo(() => {
        const seen = new Set<string>();
        const out: string[] = [];
        for (const t of [...allTags, ...tags]) {
            const k = tagKey(t);
            if (!k || seen.has(k)) continue;
            seen.add(k);
            out.push(t.trim());
        }
        return out.sort((a, b) => a.localeCompare(b));
    }, [allTags, tags]);

    const query = tagKey(input);
    const filtered = query ? candidates.filter(t => t.toLowerCase().includes(query)) : candidates;

    const isAdded = (tag: string) => tags.some(t => tagKey(t) === tagKey(tag));

    const addTag = (raw: string) => {
        const tag = raw.trim();
        if (!tag) return;
        if (isAdded(tag)) {
            setHint(`「${tag}」已加入(tag 不分大小寫)`);
            return;
        }
        onTagsChange([...tags, tag]);
        setInput('');
        setHint(null);
    };

    const removeTag = (tag: string) => onTagsChange(tags.filter(t => tagKey(t) !== tagKey(tag)));

    // Tab 循環鎖在 dialog 內
    const trapFocus = (e: React.KeyboardEvent) => {
        if (e.key !== 'Tab') return;
        const focusables = dialogRef.current?.querySelectorAll<HTMLElement>('button, input');
        if (!focusables?.length) return;
        const first = focusables[0];
        const last = focusables[focusables.length - 1];
        if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last.focus(); }
        else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first.focus(); }
    };

    return (
        <div
            className="fixed inset-0 z-10 flex items-center justify-center bg-black/50 p-4"
            onClick={onClose}
        >
            <div
                ref={dialogRef}
                role="dialog"
                aria-modal="true"
                aria-label="編輯 Tag"
                onKeyDown={trapFocus}
                className="bg-white dark:bg-neutral-800 p-6 rounded-lg shadow-lg w-full max-w-md max-h-[80vh] overflow-auto"
                onClick={(e) => e.stopPropagation()}
            >
                <h2 className="text-lg font-semibold mb-1">編輯 Tag</h2>
                <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-4">
                    變更會先存為本機草稿,按「存檔」後才寫入文章。
                </p>

                <div className="flex flex-wrap gap-2 mb-4">
                    {tags.length === 0 && (
                        <span className="text-sm text-neutral-400">尚未加入任何 tag</span>
                    )}
                    {tags.map((tag) => (
                        <span
                            key={tag}
                            className="flex items-center gap-1 bg-primary-50 dark:bg-primary-900/40 border border-primary-200 dark:border-primary-800 rounded-lg pl-2.5 pr-1 py-0.5 text-sm text-neutral-700 dark:text-neutral-200"
                        >
                            {tag}
                            <button
                                onClick={() => removeTag(tag)}
                                aria-label={`移除 ${tag}`}
                                className="p-0.5 rounded text-neutral-400 hover:text-red-600 transition-colors"
                            >
                                <X size={14} />
                            </button>
                        </span>
                    ))}
                </div>

                <div className="flex gap-2 mb-1">
                    <input
                        ref={inputRef}
                        type="text"
                        value={input}
                        onChange={(e) => { setInput(e.target.value); setHint(null); }}
                        onKeyDown={(e) => { if (e.key === 'Enter' && !e.nativeEvent.isComposing) addTag(input); }}
                        className="flex-1 min-w-0 p-2 border rounded dark:bg-neutral-700 dark:border-neutral-600"
                        placeholder="搜尋或輸入新 tag..."
                    />
                    <button
                        onClick={() => addTag(input)}
                        disabled={!input.trim()}
                        className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:bg-neutral-400 disabled:cursor-not-allowed transition-colors"
                    >
                        新增
                    </button>
                </div>
                {hint && <p className="text-xs text-red-500 mb-1">{hint}</p>}

                <h3 className="text-sm font-medium mt-4 mb-2 text-neutral-600 dark:text-neutral-300">
                    所有 Tag(點擊加入或移除)
                </h3>
                <div className="flex flex-wrap gap-2 max-h-48 overflow-auto border-t border-neutral-200 dark:border-neutral-700 pt-3">
                    {filtered.length === 0 && (
                        <span className="text-sm text-neutral-400">沒有符合的 tag,可直接新增</span>
                    )}
                    {filtered.map((tag) => {
                        const added = isAdded(tag);
                        return (
                            <button
                                key={tag}
                                onClick={() => (added ? removeTag(tag) : addTag(tag))}
                                aria-pressed={added}
                                className={`flex items-center gap-1 px-2.5 py-0.5 text-sm rounded-lg border transition-colors ${added
                                    ? 'bg-primary-600 border-primary-600 text-white'
                                    : 'bg-white dark:bg-neutral-700 border-neutral-300 dark:border-neutral-600 text-neutral-700 dark:text-neutral-200 hover:border-primary-400'}`}
                            >
                                {added && <Check size={14} />}
                                {tag}
                            </button>
                        );
                    })}
                </div>

                <button
                    onClick={onClose}
                    className="mt-4 w-full px-4 py-2 bg-neutral-600 text-white rounded-lg hover:bg-neutral-700 transition-colors"
                >
                    完成
                </button>
            </div>
        </div>
    );
}
