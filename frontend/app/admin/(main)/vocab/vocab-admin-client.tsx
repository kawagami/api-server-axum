"use client";

import { useCallback, useEffect, useState } from "react";
import { Loader2, Pencil } from "lucide-react";
import { getAdminVocabWords, updateAdminVocabWord } from "@/api/admin-vocab";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import type { AdminVocabWord, UpdateVocabWordInput } from "@/types";

const LIMIT = 50;

interface Filters {
    language: string;   // '' | 'en' | 'ja'
    difficulty: string; // '' | '1'..'5'
    enabled: string;    // '' | 'true' | 'false'
    q: string;
    wrongFirst: boolean;
}

const defaultFilters: Filters = { language: '', difficulty: '', enabled: '', q: '', wrongFirst: false };

function toParams(f: Filters) {
    return {
        language: f.language || undefined,
        difficulty: f.difficulty ? Number(f.difficulty) : undefined,
        enabled: f.enabled ? f.enabled === 'true' : undefined,
        q: f.q || undefined,
        sort: f.wrongFirst ? 'wrong' : undefined,
    };
}

export default function VocabAdminClient({ canUpdate }: { canUpdate: boolean }) {
    const { items: words, setItems, hasMore, isPending, load, loadMore } = usePagedList<AdminVocabWord>(LIMIT);
    const [filters, setFilters] = useState<Filters>(defaultFilters);
    const [total, setTotal] = useState(0);
    const [editing, setEditing] = useState<AdminVocabWord | null>(null);
    const [savingId, setSavingId] = useState<number | null>(null);

    const search = useCallback((f: Filters) => {
        load(async page => {
            const res = await getAdminVocabWords({ ...toParams(f), page, per_page: LIMIT });
            setTotal(res.total);
            return res.data;
        });
    }, [load]);

    useEffect(() => { search(defaultFilters); }, [search]);

    function applyLocal(updated: AdminVocabWord) {
        setItems(prev => prev.map(w => (w.id === updated.id ? updated : w)));
    }

    /** 快速上/下架:其餘欄位照列上現值送全欄位覆寫 */
    async function toggleEnabled(w: AdminVocabWord) {
        if (savingId) return;
        setSavingId(w.id);
        try {
            await updateAdminVocabWord(w.id, { ...pickUpdate(w), enabled: !w.enabled });
            applyLocal({ ...w, enabled: !w.enabled });
        } catch {
            // adminRequest 已處理 401;其餘錯誤維持原狀即可看出未生效
        } finally {
            setSavingId(null);
        }
    }

    const inputClass = "px-2 py-1.5 text-sm rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-1 focus:ring-primary-400";

    return (
        <div className="w-full p-3 sm:p-6">
            <div className="flex flex-col gap-4">
                <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">
                    單字題庫
                    <span className="ml-2 text-sm font-normal text-neutral-400">共 {total} 字</span>
                </h1>

                {/* Filter bar */}
                <div className="flex flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">語言</label>
                        <select value={filters.language}
                            onChange={e => setFilters(f => ({ ...f, language: e.target.value }))}
                            className={inputClass}>
                            <option value="">全部</option>
                            <option value="en">英文</option>
                            <option value="ja">日文</option>
                        </select>
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">難度</label>
                        <select value={filters.difficulty}
                            onChange={e => setFilters(f => ({ ...f, difficulty: e.target.value }))}
                            className={inputClass}>
                            <option value="">全部</option>
                            {[1, 2, 3, 4, 5].map(d => <option key={d} value={d}>{d}</option>)}
                        </select>
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">狀態</label>
                        <select value={filters.enabled}
                            onChange={e => setFilters(f => ({ ...f, enabled: e.target.value }))}
                            className={inputClass}>
                            <option value="">全部</option>
                            <option value="true">上架</option>
                            <option value="false">下架</option>
                        </select>
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">表記 / 讀音 / 釋義</label>
                        <input type="text" value={filters.q}
                            onChange={e => setFilters(f => ({ ...f, q: e.target.value }))}
                            onKeyDown={e => e.key === 'Enter' && search(filters)}
                            placeholder="食べる" className={`${inputClass} w-40`} />
                    </div>
                    <label className="flex items-center gap-1.5 pb-1.5 text-sm text-neutral-600 dark:text-neutral-300 cursor-pointer">
                        <input type="checkbox" checked={filters.wrongFirst}
                            onChange={e => setFilters(f => ({ ...f, wrongFirst: e.target.checked }))}
                            className="accent-primary-500" />
                        錯最多優先
                    </label>
                    <div className="flex gap-2">
                        <button onClick={() => search(filters)} disabled={isPending}
                            className="px-4 py-1.5 text-sm font-medium rounded bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50 transition-colors">
                            搜尋
                        </button>
                        <button onClick={() => { setFilters(defaultFilters); search(defaultFilters); }} disabled={isPending}
                            className="px-4 py-1.5 text-sm font-medium rounded bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors">
                            重設
                        </button>
                    </div>
                </div>

                <div className={`bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                    <div className="overflow-x-auto">
                        <AdminTable className="text-sm">
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh className="whitespace-nowrap">語言</AdminTh>
                                    <AdminTh className="whitespace-nowrap">表記</AdminTh>
                                    <AdminTh className="whitespace-nowrap">讀音</AdminTh>
                                    <AdminTh>釋義</AdminTh>
                                    <AdminTh className="whitespace-nowrap">詞性</AdminTh>
                                    <AdminTh className="whitespace-nowrap">難度</AdminTh>
                                    <AdminTh className="whitespace-nowrap" title="全會員 答錯/答對 次數">✗/✓</AdminTh>
                                    <AdminTh className="whitespace-nowrap">狀態</AdminTh>
                                    {canUpdate && <AdminTh className="whitespace-nowrap">操作</AdminTh>}
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {words.length === 0 ? (
                                    <tr>
                                        <td colSpan={canUpdate ? 9 : 8} className="px-4 py-8 text-center text-neutral-500 dark:text-neutral-400">
                                            {isPending ? '載入中…' : '沒有符合條件的單字'}
                                        </td>
                                    </tr>
                                ) : (
                                    words.map(w => (
                                        <AdminRow key={w.id} className={w.enabled ? '' : 'opacity-50'}>
                                            <AdminTd className="whitespace-nowrap text-xs">{w.language}</AdminTd>
                                            <AdminTd className="whitespace-nowrap font-semibold"
                                                lang={w.language === 'ja' ? 'ja' : undefined}>{w.word}</AdminTd>
                                            <AdminTd className="whitespace-nowrap text-neutral-500 dark:text-neutral-400"
                                                lang={w.language === 'ja' ? 'ja' : undefined}>{w.reading ?? '—'}</AdminTd>
                                            <AdminTd className="max-w-xs truncate">{w.meaning_zh}</AdminTd>
                                            <AdminTd className="whitespace-nowrap text-xs">{w.part_of_speech}</AdminTd>
                                            <AdminTd className="whitespace-nowrap text-center">{w.difficulty}</AdminTd>
                                            <AdminTd className="whitespace-nowrap text-xs">
                                                <span className="text-red-500">✗{w.wrong_total}</span>
                                                <span className="ml-1 text-green-600 dark:text-green-400">✓{w.correct_total}</span>
                                            </AdminTd>
                                            <AdminTd className="whitespace-nowrap text-xs">
                                                {w.enabled
                                                    ? <span className="text-green-600 dark:text-green-400">上架</span>
                                                    : <span className="text-neutral-400">下架</span>}
                                            </AdminTd>
                                            {canUpdate && (
                                                <AdminTd className="whitespace-nowrap">
                                                    <div className="flex items-center gap-2">
                                                        <button onClick={() => setEditing(w)}
                                                            className="p-1 rounded text-neutral-400 hover:text-primary-500 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
                                                            title="編輯" aria-label={`編輯 ${w.word}`}>
                                                            <Pencil size={15} />
                                                        </button>
                                                        <button onClick={() => toggleEnabled(w)} disabled={savingId === w.id}
                                                            className="px-2 py-0.5 text-xs rounded border border-neutral-300 dark:border-neutral-600 hover:border-primary-400 disabled:opacity-50 transition-colors">
                                                            {savingId === w.id
                                                                ? <Loader2 size={12} className="animate-spin" />
                                                                : w.enabled ? '下架' : '上架'}
                                                        </button>
                                                    </div>
                                                </AdminTd>
                                            )}
                                        </AdminRow>
                                    ))
                                )}
                            </tbody>
                        </AdminTable>
                    </div>
                </div>

                {hasMore && (
                    <button onClick={loadMore} disabled={isPending}
                        className="self-center px-6 py-2 text-sm font-medium rounded bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors">
                        {isPending ? '載入中…' : '載入更多'}
                    </button>
                )}
            </div>

            {editing && (
                <EditModal word={editing} onClose={() => setEditing(null)}
                    onSaved={w => { applyLocal(w); setEditing(null); }} />
            )}
        </div>
    );
}

/** 列資料 → 全欄位覆寫 payload */
function pickUpdate(w: AdminVocabWord): UpdateVocabWordInput {
    return {
        reading: w.reading,
        accepted_readings: w.accepted_readings,
        part_of_speech: w.part_of_speech,
        meaning_zh: w.meaning_zh,
        example_sentence: w.example_sentence,
        difficulty: w.difficulty,
        enabled: w.enabled,
    };
}

function EditModal({ word, onClose, onSaved }: {
    word: AdminVocabWord;
    onClose: () => void;
    onSaved: (w: AdminVocabWord) => void;
}) {
    const ja = word.language === 'ja';
    const [form, setForm] = useState({
        reading: word.reading ?? '',
        // 可接受讀音以「|」分隔編輯(與匯入腳本 TSV 同慣例)
        accepted: (word.accepted_readings ?? []).join('|'),
        part_of_speech: word.part_of_speech,
        meaning_zh: word.meaning_zh,
        example_sentence: word.example_sentence,
        difficulty: word.difficulty,
        enabled: word.enabled,
    });
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState('');

    async function save() {
        if (saving) return;
        if (!form.meaning_zh.trim()) { setError('釋義不可為空'); return; }
        if (ja && !form.reading.trim()) { setError('日文單字必須有讀音'); return; }
        setSaving(true);
        setError('');
        const accepted = form.accepted.split('|').map(s => s.trim()).filter(Boolean);
        const input: UpdateVocabWordInput = {
            reading: ja ? form.reading.trim() : (form.reading.trim() || null),
            accepted_readings: ja ? accepted : (accepted.length ? accepted : null),
            part_of_speech: form.part_of_speech.trim(),
            meaning_zh: form.meaning_zh.trim(),
            example_sentence: form.example_sentence,
            difficulty: form.difficulty,
            enabled: form.enabled,
        };
        try {
            await updateAdminVocabWord(word.id, input);
            // 後端會把主讀音補進 accepted;本地同步同一規則,顯示才一致
            const savedAccepted = ja && input.reading && !accepted.includes(input.reading)
                ? [input.reading, ...accepted] : (input.accepted_readings ?? null);
            onSaved({ ...word, ...input, accepted_readings: savedAccepted });
        } catch (e) {
            setError((e as Error).message || '儲存失敗');
            setSaving(false);
        }
    }

    const fieldClass = "w-full px-2 py-1.5 text-sm rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-1 focus:ring-primary-400";

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4" onClick={onClose}>
            <div className="w-full max-w-md bg-white dark:bg-neutral-900 rounded-lg shadow-xl p-5 flex flex-col gap-3"
                onClick={e => e.stopPropagation()}>
                <h2 className="font-semibold text-neutral-800 dark:text-neutral-100">
                    編輯:<span lang={ja ? 'ja' : undefined}>{word.word}</span>
                    <span className="ml-2 text-xs font-normal text-neutral-400">{word.language} / id {word.id}</span>
                </h2>
                {ja && (
                    <>
                        <Field label="讀音(平假名)">
                            <input value={form.reading} lang="ja"
                                onChange={e => setForm(f => ({ ...f, reading: e.target.value }))} className={fieldClass} />
                        </Field>
                        <Field label="可接受讀音(以 | 分隔;主讀音會自動補入)">
                            <input value={form.accepted} lang="ja"
                                onChange={e => setForm(f => ({ ...f, accepted: e.target.value }))} className={fieldClass} />
                        </Field>
                    </>
                )}
                <Field label="詞性">
                    <input value={form.part_of_speech}
                        onChange={e => setForm(f => ({ ...f, part_of_speech: e.target.value }))} className={fieldClass} />
                </Field>
                <Field label="中文釋義">
                    <input value={form.meaning_zh}
                        onChange={e => setForm(f => ({ ...f, meaning_zh: e.target.value }))} className={fieldClass} />
                </Field>
                <Field label="例句(拼字題挖空用;日文目前未使用)">
                    <textarea value={form.example_sentence} rows={2}
                        onChange={e => setForm(f => ({ ...f, example_sentence: e.target.value }))} className={fieldClass} />
                </Field>
                <div className="flex items-center gap-4">
                    <Field label="難度">
                        <select value={form.difficulty}
                            onChange={e => setForm(f => ({ ...f, difficulty: Number(e.target.value) }))} className={fieldClass}>
                            {[1, 2, 3, 4, 5].map(d => <option key={d} value={d}>{d}</option>)}
                        </select>
                    </Field>
                    <label className="flex items-center gap-1.5 pt-4 text-sm text-neutral-600 dark:text-neutral-300 cursor-pointer whitespace-nowrap">
                        <input type="checkbox" checked={form.enabled}
                            onChange={e => setForm(f => ({ ...f, enabled: e.target.checked }))}
                            className="accent-primary-500" />
                        上架
                    </label>
                </div>
                {error && <p className="text-sm text-red-500">{error}</p>}
                <div className="flex justify-end gap-2 pt-1">
                    <button onClick={onClose} disabled={saving}
                        className="px-4 py-1.5 text-sm rounded bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600 transition-colors">
                        取消
                    </button>
                    <button onClick={save} disabled={saving}
                        className="px-4 py-1.5 text-sm font-medium rounded bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50 transition-colors flex items-center gap-1">
                        {saving && <Loader2 size={14} className="animate-spin" />}儲存
                    </button>
                </div>
            </div>
        </div>
    );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
    return (
        <div className="flex flex-col gap-1 flex-1">
            <label className="text-xs text-neutral-500 dark:text-neutral-400">{label}</label>
            {children}
        </div>
    );
}
