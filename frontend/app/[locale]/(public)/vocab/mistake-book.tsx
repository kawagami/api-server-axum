"use client";

import type { VocabMistake, VocabMistakeSort, VocabMistakesPage } from "@/types";
import { BookOpenCheck, CheckCircle2, Loader2, Search } from "lucide-react";
import type { MistakeQuery } from "./model";
import { SpeakButton, type T } from "./ui";

const SORTS: VocabMistakeSort[] = ["wrong", "recent", "difficulty", "word"];
const SORT_KEYS: Record<VocabMistakeSort, "msWrong" | "msRecent" | "msDifficulty" | "msWord"> = {
    wrong: "msWrong", recent: "msRecent", difficulty: "msDifficulty", word: "msWord",
};

export function MistakeBook({ page, query, searchInput, loading, error, canTts, ja, onSearch, onQuery, onMore, onSpeak, t }: {
    page: VocabMistakesPage | null; query: MistakeQuery; searchInput: string;
    loading: boolean; error: boolean; canTts: boolean; ja: boolean;
    onSearch: (v: string) => void; onQuery: (patch: Partial<MistakeQuery>) => void;
    onMore: () => void; onSpeak: (text: string) => void; t: T;
}) {
    const items = page?.items ?? [];
    const total = page?.total ?? 0;
    const filtering = query.q !== "" || query.unmastered;
    // page 為 null = 從沒拿到資料(SSR 那趟就掛了),不能當成「沒錯過字」
    if (!page) {
        return (
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex justify-center text-sm text-red-500">
                {loading
                    ? <Loader2 size={20} className="animate-spin text-neutral-400" />
                    : t("mistakeError")}
            </div>
        );
    }
    // 完全沒有錯字(非搜尋造成的空)才顯示鼓勵文案,不然搜不到會被誤讀成「沒錯過字」
    if (total === 0 && !filtering && !loading) {
        return (
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm text-center text-sm text-neutral-500 dark:text-neutral-400">
                {t("mistakeEmpty")}
            </div>
        );
    }
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow-sm flex flex-col gap-3">
            <div className="flex items-center justify-between px-2">
                <h2 className="font-bold flex items-center gap-1">
                    <BookOpenCheck size={18} className="text-primary-500" aria-hidden />{t("mistakeBook")}
                </h2>
                <span className="text-xs text-neutral-400 dark:text-neutral-500">{t("mistakeCount", { count: total })}</span>
            </div>

            <div className="flex flex-wrap items-center gap-2 px-2">
                <label className="relative flex-1 min-w-40">
                    <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400" aria-hidden />
                    <span className="sr-only">{t("mistakeSearch")}</span>
                    <input value={searchInput} onChange={e => onSearch(e.target.value)}
                        placeholder={t("mistakeSearch")} autoComplete="off"
                        className="w-full pl-8 pr-3 py-1.5 rounded-lg border border-neutral-200 dark:border-neutral-600 bg-transparent text-sm" />
                </label>
                <label className="flex items-center gap-1 text-xs text-neutral-500 dark:text-neutral-400">
                    <span>{t("mistakeSortLabel")}</span>
                    <select value={query.sort} onChange={e => onQuery({ sort: e.target.value as VocabMistakeSort })}
                        className="px-2 py-1.5 rounded-lg border border-neutral-200 dark:border-neutral-600 bg-transparent text-sm">
                        {SORTS.map(s => <option key={s} value={s}>{t(SORT_KEYS[s])}</option>)}
                    </select>
                </label>
                <label className="flex items-center gap-1 text-xs text-neutral-500 dark:text-neutral-400">
                    <input type="checkbox" checked={query.unmastered}
                        onChange={e => onQuery({ unmastered: e.target.checked })}
                        className="accent-primary-500" />
                    {t("mistakeUnmastered")}
                </label>
            </div>

            {error && <p className="text-center text-sm text-red-500">{t("mistakeError")}</p>}

            {items.length === 0 ? (
                loading
                    ? <div className="flex justify-center py-6"><Loader2 size={20} className="animate-spin text-neutral-400" /></div>
                    : <p className="text-center text-sm text-neutral-500 dark:text-neutral-400 py-4">{t("mistakeNoMatch")}</p>
            ) : (
                <div className="flex flex-col divide-y divide-neutral-100 dark:divide-neutral-700">
                    {items.map(m => <MistakeRow key={`${m.word}|${m.reading ?? ""}`} m={m}
                        canTts={canTts} ja={ja} onSpeak={onSpeak} t={t} />)}
                </div>
            )}

            {items.length < total && (
                <button onClick={onMore} disabled={loading}
                    className="mx-auto px-4 py-1.5 rounded-lg border border-neutral-200 dark:border-neutral-600 text-sm hover:border-primary-400 transition-colors disabled:opacity-50 flex items-center gap-2">
                    {loading && <Loader2 size={14} className="animate-spin" />}
                    {t("loadMore", { count: total - items.length })}
                </button>
            )}
        </div>
    );
}

function MistakeRow({ m, canTts, ja, onSpeak, t }: {
    m: VocabMistake; canTts: boolean; ja: boolean; onSpeak: (text: string) => void; t: T;
}) {
    const mastered = m.correct_count >= m.wrong_count;
    // 日文唸讀音、英文唸表記
    const speakText = ja ? (m.reading ?? m.word) : m.word;
    return (
        <div className="flex items-center gap-3 py-2 px-2">
            <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                    {/* 日文同表記可有多讀音(辛い=からい/つらい),所以列 key 含讀音 */}
                    <span className={`font-semibold truncate ${m.reading ? "font-ja" : ""}`}
                        lang={m.reading ? "ja" : undefined}>{m.word}</span>
                    {m.reading && (
                        <span className="text-xs text-neutral-400 dark:text-neutral-500 shrink-0 font-ja" lang="ja">
                            {m.reading}
                        </span>
                    )}
                    <SpeakButton text={speakText} canTts={canTts} onSpeak={onSpeak} t={t} size={14} />
                    <span className="text-xs text-neutral-400 dark:text-neutral-500 shrink-0">{m.part_of_speech}</span>
                </div>
                <div className="text-sm text-neutral-500 dark:text-neutral-400 truncate">{m.meaning_zh}</div>
            </div>
            <div className="flex items-center gap-2 shrink-0">
                <span className="text-xs text-red-500" title={t("wrongCount")}>
                    <span className="sr-only">{t("wrongCount")}</span>✗{m.wrong_count}
                </span>
                <span className="text-xs text-green-600 dark:text-green-400" title={t("correctCount")}>
                    <span className="sr-only">{t("correctCount")}</span>✓{m.correct_count}
                </span>
                {mastered && <CheckCircle2 size={16} className="text-green-500" aria-label={t("mastered")} />}
            </div>
        </div>
    );
}
