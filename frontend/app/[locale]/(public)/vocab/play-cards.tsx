"use client";

import type { VocabQuestion, VocabRunMode } from "@/types";
import { BookOpenCheck, Check, Clock, Flame, Heart, X } from "lucide-react";
import { type Feedback, diffMarks, hasLives, hasTimer } from "./model";
import { SpeakButton, type T, fmtTime } from "./ui";

/** 連對分級:5 / 10 / 20 連各換一次火焰樣式 */
const COMBO_TIER = [
    "text-primary-600 dark:text-primary-400",
    "text-orange-500 scale-110",
    "text-orange-500 scale-125 drop-shadow-[0_0_6px_rgba(249,115,22,0.7)]",
    "text-red-500 scale-125 drop-shadow-[0_0_10px_rgba(239,68,68,0.8)] animate-pulse",
];
function comboTier(combo: number) {
    return combo >= 20 ? 3 : combo >= 10 ? 2 : combo >= 5 ? 1 : 0;
}

export function PlayHeader({ mode, lives, combo, runExp, number, remaining, timeTotal, t }: {
    mode: VocabRunMode; lives: number; combo: number; runExp: number; number: number; remaining: number; timeTotal: number; t: T;
}) {
    const timePct = timeTotal > 0 ? Math.min(100, (remaining / timeTotal) * 100) : 0;
    return (
        <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-3">
                {hasLives(mode) && (
                    <div className="flex gap-1" role="img" aria-label={t("livesLeft", { count: lives })}>
                        {/* key 帶滿／空:扣命那一顆換元素,碎裂動畫只播一次 */}
                        {[0, 1, 2].map(i => i < lives
                            ? <Heart key={`${i}f`} size={20} aria-hidden className="text-red-500 fill-red-500" />
                            : <Heart key={`${i}e`} size={20} aria-hidden className="fx-heart-break text-neutral-300 dark:text-neutral-600" />)}
                    </div>
                )}
                {hasTimer(mode) && (
                    // aria-live 刻意不開:每秒播報剩餘時間對讀屏是噪音
                    <span className={`flex items-center gap-1 font-mono font-semibold text-sm ${remaining <= 30 ? "text-red-500" : "text-neutral-600 dark:text-neutral-300"}`}>
                        <Clock size={16} aria-hidden />
                        <span className="sr-only">{t("timeRemainingLabel")}</span>
                        {fmtTime(remaining)}
                    </span>
                )}
            </div>
            <span className="text-sm font-medium text-neutral-500 dark:text-neutral-400">
                {t("questionNumber", { number })}
            </span>
            <div className="flex items-center gap-3">
                {combo > 1 && (
                    <span className={`flex items-center gap-1 font-semibold text-sm transition-transform ${COMBO_TIER[comboTier(combo)]}`}>
                        <Flame size={16} aria-hidden className={comboTier(combo) >= 2 ? "fill-current" : ""} />{t("comboLabel", { count: combo })}
                    </span>
                )}
                <span className="text-sm font-semibold">{runExp} EXP</span>
            </div>
        </div>
        {/* 倒數條:最後 30 秒變紅閃爍 */}
        {hasTimer(mode) && timeTotal > 0 && (
            <div className="h-1.5 rounded-full bg-neutral-100 dark:bg-neutral-700 overflow-hidden" aria-hidden>
                <div className={`h-full rounded-full transition-[width] duration-500 ease-linear ${remaining <= 30 ? "bg-red-500 animate-pulse" : "bg-primary-500"}`}
                    style={{ width: `${timePct}%` }} />
            </div>
        )}
        </div>
    );
}

export function ReviewHeader({ number, total, t }: { number: number; total: number; t: T }) {
    const progress = total > 0 ? Math.min(100, ((number - 1) / total) * 100) : 0;
    return (
        <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between">
                <span className="flex items-center gap-1 text-primary-600 dark:text-primary-400 font-semibold text-sm">
                    <BookOpenCheck size={16} aria-hidden />{t("reviewMode")}
                </span>
                <span className="text-sm font-medium text-neutral-500 dark:text-neutral-400">{number} / {total}</span>
            </div>
            <div className="h-2 rounded-full bg-neutral-100 dark:bg-neutral-700 overflow-hidden">
                <div className="h-full rounded-full bg-primary-500 transition-all" style={{ width: `${progress}%` }} />
            </div>
        </div>
    );
}

function DifficultyDots({ difficulty, t }: { difficulty: number; t: T }) {
    return (
        <div className="flex gap-1">
            <span className="sr-only">{t("difficultyLabel", { n: difficulty })}</span>
            {[1, 2, 3, 4, 5].map(i => (
                <span key={i} aria-hidden
                    className={`w-1.5 h-1.5 rounded-full ${i <= difficulty ? "bg-primary-400" : "bg-neutral-200 dark:bg-neutral-600"}`} />
            ))}
        </div>
    );
}

/** 答對/答錯的播報區。role="status" 讓讀屏在不搶焦點的情況下唸出結果。 */
function FeedbackBanner({ feedback, t }: { feedback: Feedback; t: T }) {
    return (
        <div role="status" aria-live="polite"
            className={`text-center font-semibold ${feedback.correct ? "text-green-600 dark:text-green-400" : "text-red-500"}`}>
            {feedback.correct
                ? <>{t("correct")}{feedback.gainedExp > 0 && <span className="ml-1">+{feedback.gainedExp} EXP</span>}</>
                : t("wrong")}
        </div>
    );
}

function ContinueHint({ t }: { t: T }) {
    return <p className="text-xs text-neutral-400 dark:text-neutral-500">{t("continueHint")}</p>;
}

export function ChoiceCard({ question, feedback, busy, ja, canTts, onPick, onSpeak, t }: {
    question: VocabQuestion; feedback: Feedback | null; busy: boolean; ja: boolean; canTts: boolean;
    onPick: (i: number) => void; onSpeak: (text: string) => void; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex flex-col gap-5">
            <div className="flex flex-col items-center gap-2">
                <DifficultyDots difficulty={question.difficulty} t={t} />
                <div className="flex items-center gap-1">
                    {/* lang="ja" + 日文字型:避免瀏覽器用中文字形渲染日文漢字 */}
                    <span className={`text-3xl font-bold tracking-wide ${ja ? "font-ja" : ""}`}
                        lang={ja ? "ja" : undefined}>{question.word}</span>
                    <SpeakButton text={question.word} canTts={canTts} onSpeak={onSpeak} t={t} size={18} />
                </div>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">{question.part_of_speech}</span>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">{t("chooseMeaning")}</span>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {question.options?.map((opt, i) => {
                    const isAnswer = feedback && i === feedback.correctChoiceIndex;
                    const isWrongPick = feedback && !isAnswer && i === feedback.selectedIndex;
                    let cls = "border-neutral-200 dark:border-neutral-600 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950";
                    if (feedback) {
                        if (isAnswer) cls = "border-green-500 bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300";
                        else if (isWrongPick) cls = "border-red-500 bg-red-50 dark:bg-red-950 text-red-600 dark:text-red-300";
                        else cls = "border-neutral-200 dark:border-neutral-600 opacity-60";
                    }
                    return (
                        <button key={i} onClick={() => onPick(i)} disabled={busy || !!feedback}
                            className={`flex items-center gap-2 px-4 py-3 rounded-lg border text-left transition-colors ${cls}`}>
                            {/* 數字對應鍵盤 1–4,不是裝飾 */}
                            <kbd className="shrink-0 w-5 h-5 rounded border border-current/30 text-[11px] font-mono flex items-center justify-center opacity-60">
                                {i + 1}
                            </kbd>
                            <span className="flex-1 min-w-0">{opt}</span>
                            {/* 對錯不能只靠顏色 */}
                            {isAnswer && <Check size={16} className="shrink-0" aria-label={t("correctCount")} />}
                            {isWrongPick && <X size={16} className="shrink-0" aria-label={t("wrongCount")} />}
                        </button>
                    );
                })}
            </div>
            {feedback && (
                <div className="flex flex-col items-center gap-1">
                    <FeedbackBanner feedback={feedback} t={t} />
                    {/* 日文:答後回饋讀音(題面不顯示 furigana,避免白給讀音) */}
                    {ja && feedback.reading && (
                        <span className="flex items-center gap-1 text-sm text-neutral-500 dark:text-neutral-400 font-ja" lang="ja">
                            {t("readingIs", { reading: feedback.reading })}
                            <SpeakButton text={feedback.reading} canTts={canTts} onSpeak={onSpeak} t={t} />
                        </span>
                    )}
                    <ContinueHint t={t} />
                </div>
            )}
        </div>
    );
}

export function SpellingCard({ question, feedback, busy, ja, canTts, value, onChange, onSubmit, onSpeak, inputRef, composingRef, t }: {
    question: VocabQuestion; feedback: Feedback | null; busy: boolean; ja: boolean; canTts: boolean;
    value: string; onChange: (v: string) => void; onSubmit: () => void; onSpeak: (text: string) => void;
    inputRef: React.RefObject<HTMLInputElement | null>;
    composingRef: React.RefObject<boolean>; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex flex-col gap-5">
            <div className="flex flex-col items-center gap-2">
                <DifficultyDots difficulty={question.difficulty} t={t} />
                <span className="text-2xl font-bold">{question.meaning_zh}</span>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">{question.part_of_speech}</span>
                {question.sentence_masked && (
                    <p className="text-neutral-600 dark:text-neutral-300 text-center font-mono text-sm bg-neutral-50 dark:bg-neutral-700/50 rounded-lg px-4 py-3 w-full">
                        {question.sentence_masked}
                    </p>
                )}
                <span className="text-xs text-neutral-400 dark:text-neutral-500">
                    {t(ja ? "spellingHintJa" : "spellingHint", { letter: question.hint_first_letter ?? "?", length: question.hint_length ?? 0 })}
                </span>
                {/* 英文才畫字母格:日文的 hint_length 是「拍數」,拗音（きゃ）兩個字算一拍,格數會對不上 */}
                {!ja && !feedback && (question.hint_length ?? 0) > 0 && (
                    <LetterSlots value={value} length={question.hint_length ?? 0} hint={question.hint_first_letter}
                        onClick={() => inputRef.current?.focus()} />
                )}
            </div>
            <form className="flex gap-2" onSubmit={(e) => { e.preventDefault(); onSubmit(); }}>
                <input ref={inputRef} value={value} onChange={(e) => onChange(e.target.value)}
                    disabled={busy || !!feedback} placeholder={t(ja ? "inputPlaceholderJa" : "inputPlaceholder")}
                    autoComplete="off" autoCapitalize="off" spellCheck={false}
                    lang={ja ? "ja" : undefined}
                    // IME 組字中的 Enter 是選字確認,不能觸發送出
                    onKeyDown={(e) => { if (e.key === "Enter" && e.nativeEvent.isComposing) e.preventDefault(); }}
                    onCompositionStart={() => { composingRef.current = true; }}
                    onCompositionEnd={(e) => { composingRef.current = false; onChange(e.currentTarget.value); }}
                    className={`flex-1 px-4 py-2 rounded-lg border border-neutral-200 dark:border-neutral-600 bg-transparent ${ja ? "font-ja" : "font-mono"}`} />
                <button type="submit" disabled={busy || !!feedback || !value.trim()}
                    className="px-5 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50">
                    {t("submit")}
                </button>
            </form>
            {feedback && (
                <div className="flex flex-col items-center gap-1">
                    <FeedbackBanner feedback={feedback} t={t} />
                    {feedback.correctText && <AnswerDiff feedback={feedback} ja={ja} t={t} />}
                    {feedback.correctText && (
                        <span className={`flex items-center gap-1 text-sm text-neutral-500 dark:text-neutral-400 ${ja ? "font-ja" : ""}`}
                            lang={ja ? "ja" : undefined}>
                            {feedback.correct
                                ? feedback.correctText
                                : t("correctAnswerIs", { answer: feedback.correctText })}
                            <SpeakButton text={feedback.correctText} canTts={canTts} onSpeak={onSpeak} t={t} />
                        </span>
                    )}
                    <ContinueHint t={t} />
                </div>
            )}
        </div>
    );
}

/** 拼字輸入的字母格(英文):逐格顯示已打的字,第一格空著時淡淡標出提示字母,超出長度的格標紅 */
function LetterSlots({ value, length, hint, onClick }: { value: string; length: number; hint?: string; onClick: () => void }) {
    const chars = Array.from(value);
    const count = Math.max(length, chars.length);
    return (
        <div className="flex flex-wrap justify-center gap-1 cursor-text" onClick={onClick} aria-hidden>
            {Array.from({ length: count }, (_, i) => {
                const ch = chars[i];
                const over = i >= length;
                const cursor = i === chars.length;
                return (
                    <span key={i} className={`flex h-9 w-7 items-center justify-center rounded-md border-2 font-mono text-lg font-semibold sm:h-10 sm:w-8 ${over
                        ? "border-red-400 text-red-500"
                        : cursor ? "border-primary-500" : ch ? "border-neutral-400 dark:border-neutral-500" : "border-neutral-200 dark:border-neutral-600"}`}>
                        {ch ?? (i === 0 && hint ? <span className="text-neutral-300 dark:text-neutral-600">{hint}</span> : "")}
                    </span>
                );
            })}
        </div>
    );
}

/** 作答後的逐字對照:答對整排翻綠;答錯分兩排,你的答案標出打錯的字、正解標出漏掉的字 */
function AnswerDiff({ feedback, ja, t }: { feedback: Feedback; ja: boolean; t: T }) {
    const correct = feedback.correctText ?? "";
    const tile = "flex h-8 min-w-7 items-center justify-center rounded-md border-2 px-1 text-base font-semibold sm:h-9 sm:min-w-8";
    const font = ja ? "font-ja" : "font-mono";
    if (feedback.correct || !feedback.answer) {
        return (
            <div className={`flex flex-wrap justify-center gap-1 ${font}`} lang={ja ? "ja" : undefined} aria-hidden>
                {Array.from(correct).map((ch, i) => (
                    <span key={i} style={{ animationDelay: `${i * 50}ms`, animationFillMode: "both" }}
                        className={`piece-flip ${tile} ${feedback.correct
                            ? "border-green-500 bg-green-500 text-white"
                            : "border-neutral-300 dark:border-neutral-600"}`}>{ch}</span>
                ))}
            </div>
        );
    }
    const marks = diffMarks(feedback.answer, correct);
    const row = (text: string, ok: boolean[], bad: string) => Array.from(text).map((ch, i) => (
        <span key={i} className={`${tile} ${ok[i] ? "border-green-500/60 text-green-700 dark:text-green-300" : bad}`}>{ch === " " ? " " : ch}</span>
    ));
    return (
        <div className={`flex flex-col items-center gap-1.5 text-sm ${font}`} lang={ja ? "ja" : undefined}>
            <div className="flex flex-wrap items-center justify-center gap-1">
                <span className="mr-1 font-sans text-xs text-neutral-500 dark:text-neutral-400">{t("yourAnswer")}</span>
                {row(feedback.answer, marks.answer, "border-red-500 bg-red-50 text-red-600 line-through dark:bg-red-950 dark:text-red-300")}
            </div>
            <div className="flex flex-wrap items-center justify-center gap-1">
                <span className="mr-1 font-sans text-xs text-neutral-500 dark:text-neutral-400">{t("rightAnswer")}</span>
                {row(correct, marks.correct, "border-amber-500 bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300")}
            </div>
        </div>
    );
}
