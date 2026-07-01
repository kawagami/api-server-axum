"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { Link } from "@/i18n/navigation";
import { Loader2, CheckCircle2, Shuffle, Plus, Trash2, Keyboard, Camera } from "lucide-react";
import { postLotto } from "@/api/lotto";
import { GAME_SPECS, LOTTO_GAMES, GAME_KEY, nextDrawDate, isNoteValid, quickPick } from "@/libs/lotto";
import { parseLottoOcr } from "@/libs/lotto-ocr";
import NumberGrid from "@/components/lotto/NumberGrid";
import Balls from "@/components/lotto/Balls";
import LottoOcrScanner from "@/components/lotto/LottoOcrScanner";
import type { LottoGame, LottoNote, LottoInput } from "@/types";

type Mode = 'manual' | 'ocr';

const inputClass = "border rounded px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400";

export default function LottoRegisterClient() {
    const t = useTranslations('Lotto');

    const [game, setGame] = useState<LottoGame>('lotto649');
    // 預設帶下一期開獎日（lazy init 算台北日期，server/client 同值）；使用者可改
    const [drawDate, setDrawDate] = useState(() => nextDrawDate('lotto649'));
    // 正在編輯的這注
    const [picks, setPicks] = useState<number[]>([]);
    const [second, setSecond] = useState<number | null>(null);
    // 已加入的注
    const [notes, setNotes] = useState<LottoNote[]>([]);

    const [mode, setMode] = useState<Mode>('manual');
    // OCR 套用後的提示（偵測幾注 / 加入幾注）
    const [ocrInfo, setOcrInfo] = useState<{ detected: number; added: number } | null>(null);

    const [saving, setSaving] = useState(false);
    const [error, setError] = useState('');
    const [doneCount, setDoneCount] = useState(0); // >0 = 成功畫面

    const spec = GAME_SPECS[game];

    function changeGame(g: LottoGame) {
        if (g === game) return;
        setGame(g);
        setDrawDate(nextDrawDate(g)); // 切彩種重算下一期開獎日
        setPicks([]);
        setSecond(null);
        setNotes([]);
        setError('');
        setOcrInfo(null);
    }

    // OCR 解析結果套進選號區：合法注進購物車、需補第二區者載入選號區，使用者核對後才送出
    // 回傳 true = 有讀到注並已套用（即時掃描收到 true 即鎖定停止）；false = 沒讀到，續掃
    function tryApplyOcr(text: string): boolean {
        const result = parseLottoOcr(text, game);
        if (result.notes.length === 0) return false;
        if (result.drawDate) setDrawDate(result.drawDate);
        const valid = result.notes.filter(n => isNoteValid(game, n));
        if (valid.length) setNotes(prev => [...prev, ...valid]);
        const incomplete = result.notes.filter(n => !isNoteValid(game, n));
        if (valid.length === 0 && incomplete.length) setPicks(incomplete[0].picks); // 威力彩：載第一注待補第二區
        setOcrInfo({ detected: result.notes.length, added: valid.length });
        setError('');
        setMode('manual');
        return true;
    }

    function togglePick(n: number) {
        setError('');
        setPicks(prev => {
            if (prev.includes(n)) return prev.filter(x => x !== n);
            if (prev.length >= spec.mainCount) return prev; // 已滿，忽略
            return [...prev, n].sort((a, b) => a - b);
        });
    }

    function toggleSecond(n: number) {
        setError('');
        setSecond(prev => (prev === n ? null : n));
    }

    const currentNote: LottoNote = { picks, second: spec.hasSecond ? second : null };
    const currentComplete = isNoteValid(game, currentNote);
    const totalNotes = notes.length + (currentComplete ? 1 : 0);

    function clearCurrent() {
        setPicks([]);
        setSecond(null);
    }

    function addNote() {
        if (!currentComplete) return;
        setNotes(prev => [...prev, currentNote]);
        clearCurrent();
    }

    function fillQuickPick() {
        const n = quickPick(game);
        setPicks(n.picks);
        setSecond(n.second);
        setError('');
    }

    function removeNote(i: number) {
        setNotes(prev => prev.filter((_, idx) => idx !== i));
    }

    async function handleSubmit() {
        // 編輯中且完整的這注，一併送出
        const finalNotes = currentComplete ? [...notes, currentNote] : notes;
        if (finalNotes.length === 0) {
            setError(t('needOneNote'));
            return;
        }
        if (!drawDate) {
            setError(t('needDrawDate'));
            return;
        }
        setSaving(true);
        setError('');
        try {
            const input: LottoInput = { game, draw_date: drawDate, source: 'manual', notes: finalNotes };
            const created = await postLotto(input);
            setDoneCount(created.length);
            setNotes([]);
            clearCurrent();
        } catch (err) {
            const e2 = err as Error & { status?: number; errorData?: { message?: string } };
            setError(e2.errorData?.message || t('errorSave'));
        } finally {
            setSaving(false);
        }
    }

    function registerAnother() {
        setDoneCount(0);
        setError('');
        setOcrInfo(null);
        setDrawDate(nextDrawDate(game));
    }

    if (doneCount > 0) {
        return (
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-8 shadow border dark:border-neutral-700 flex flex-col items-center gap-4 text-center">
                <CheckCircle2 className="text-green-500" size={48} />
                <p className="font-semibold">{t('registered', { count: doneCount })}</p>
                <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('disclaimer')}</p>
                <div className="flex gap-2">
                    <button
                        onClick={registerAnother}
                        className="px-4 py-2 text-sm rounded bg-primary-500 text-white hover:bg-primary-600"
                    >
                        {t('registerAnother')}
                    </button>
                    <Link
                        href="/lotto"
                        className="px-4 py-2 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                    >
                        {t('viewMyTickets')}
                    </Link>
                </div>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-4">
            {/* 彩種 + 開獎日 */}
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700 flex flex-col gap-4">
                <div className="flex flex-col gap-1.5">
                    <span className="text-sm font-medium">{t('game')}</span>
                    <div className="grid grid-cols-2 gap-2">
                        {LOTTO_GAMES.map(g => (
                            <button
                                key={g}
                                type="button"
                                onClick={() => changeGame(g)}
                                className={`py-2.5 rounded-xl border text-sm font-medium transition-colors ${game === g
                                    ? 'border-primary-500 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300'
                                    : 'border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}`}
                            >
                                {t(GAME_KEY[g])}
                            </button>
                        ))}
                    </div>
                </div>
                <div className="flex flex-col gap-1.5">
                    <label className="text-sm font-medium">{t('drawDate')}</label>
                    <input
                        type="date"
                        value={drawDate}
                        onChange={e => setDrawDate(e.target.value)}
                        className={`${inputClass} w-fit`}
                    />
                    <span className="text-xs text-neutral-400 dark:text-neutral-500">{t('drawDateHint')}</span>
                </div>
            </div>

            {/* 輸入模式：手動選號 / 拍照辨識（實驗） */}
            <div className="grid grid-cols-2 gap-2">
                {([['manual', Keyboard], ['ocr', Camera]] as const).map(([m, Icon]) => (
                    <button
                        key={m}
                        type="button"
                        onClick={() => setMode(m)}
                        className={`flex items-center justify-center gap-1.5 py-2.5 rounded-xl border text-sm font-medium transition-colors ${mode === m
                            ? 'border-primary-500 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300'
                            : 'border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}`}
                    >
                        <Icon size={16} />
                        {t(m === 'manual' ? 'modeManual' : 'modeOcr')}
                    </button>
                ))}
            </div>

            {mode === 'ocr' ? (
                <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700 flex flex-col gap-4">
                    <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('ocrIntro')}</p>
                    <LottoOcrScanner onDecoded={tryApplyOcr} />
                    <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('ocrDisclaimer')}</p>
                </div>
            ) : (
            <>
            {ocrInfo && (
                <div className="rounded-xl border border-primary-200 dark:border-primary-800 bg-primary-50 dark:bg-primary-950 px-4 py-3 text-sm text-primary-700 dark:text-primary-300">
                    {t('ocrApplied', { detected: ocrInfo.detected, added: ocrInfo.added })}
                    <span className="block text-xs mt-1 text-primary-600/80 dark:text-primary-400/80">{t('ocrVerify')}</span>
                </div>
            )}

            {/* 選號區 */}
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700 flex flex-col gap-4">
                <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">
                        {spec.hasSecond ? t('firstZone') : t('pickHint', { count: spec.mainCount, max: spec.mainMax })}
                        <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500">
                            {t('selectedCount', { n: picks.length, total: spec.mainCount })}
                        </span>
                    </span>
                    <button
                        type="button"
                        onClick={fillQuickPick}
                        className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                    >
                        <Shuffle size={14} />
                        {t('quickPick')}
                    </button>
                </div>

                <NumberGrid max={spec.mainMax} selected={picks} onToggle={togglePick} full={picks.length >= spec.mainCount} />

                {spec.hasSecond && (
                    <div className="flex flex-col gap-2 border-t border-neutral-100 dark:border-neutral-700 pt-4">
                        <span className="text-sm font-medium">
                            {t('secondZone')}
                            <span className="ml-2 text-xs text-neutral-400 dark:text-neutral-500">{t('secondHint', { max: spec.secondMax })}</span>
                        </span>
                        <NumberGrid max={spec.secondMax} selected={second === null ? [] : [second]} onToggle={toggleSecond} />
                    </div>
                )}

                <div className="flex items-center justify-between gap-2 border-t border-neutral-100 dark:border-neutral-700 pt-4">
                    <button
                        type="button"
                        onClick={clearCurrent}
                        disabled={picks.length === 0 && second === null}
                        className="px-3 py-1.5 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700 disabled:opacity-50"
                    >
                        {t('clear')}
                    </button>
                    <button
                        type="button"
                        onClick={addNote}
                        disabled={!currentComplete}
                        className="flex items-center gap-1.5 px-4 py-1.5 text-sm rounded bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50"
                    >
                        <Plus size={15} />
                        {t('addNote')}
                    </button>
                </div>
            </div>
            </>
            )}

            {/* 已加入的注 */}
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700 flex flex-col gap-3">
                <span className="text-sm font-medium">{t('noteList', { count: notes.length })}</span>
                {notes.length === 0 ? (
                    <p className="text-sm text-neutral-400 dark:text-neutral-500">{t('noNotes')}</p>
                ) : (
                    <div className="flex flex-col gap-2">
                        {notes.map((n, i) => (
                            <div key={i} className="flex items-center gap-3 rounded-lg bg-neutral-50 dark:bg-neutral-900 px-3 py-2">
                                <span className="text-xs text-neutral-400 dark:text-neutral-500 shrink-0">{t('note', { n: i + 1 })}</span>
                                <div className="flex-1 min-w-0"><Balls main={n.picks} special={n.second} size="sm" /></div>
                                <button
                                    type="button"
                                    onClick={() => removeNote(i)}
                                    title={t('removeNote')}
                                    className="p-1.5 rounded hover:bg-neutral-100 dark:hover:bg-neutral-700 text-red-500 shrink-0"
                                >
                                    <Trash2 size={14} />
                                </button>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            {error && <p className="text-red-500 text-sm">{error}</p>}
            <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('disclaimer')}</p>

            <button
                type="button"
                onClick={handleSubmit}
                disabled={saving || totalNotes === 0}
                className="self-end flex items-center gap-2 px-5 py-2.5 text-sm rounded bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50"
            >
                {saving && <Loader2 className="animate-spin" size={15} />}
                {t('registerCount', { count: totalNotes })}
            </button>
        </div>
    );
}
