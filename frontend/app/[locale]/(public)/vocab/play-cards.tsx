"use client";

import type { VocabQuestion, VocabRunMode } from "@/types";
import { BookOpenCheck, Check, Clock, Flame, Heart, X } from "lucide-react";
import { type Feedback, hasLives, hasTimer } from "./model";
import { SpeakButton, type T, fmtTime } from "./ui";

export function PlayHeader({ mode, lives, combo, runExp, number, remaining, t }: {
    mode: VocabRunMode; lives: number; combo: number; runExp: number; number: number; remaining: number; t: T;
}) {
    return (
        <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-3">
                {hasLives(mode) && (
                    <div className="flex gap-1" role="img" aria-label={t("livesLeft", { count: lives })}>
                        {[0, 1, 2].map(i => (
                            <Heart key={i} size={20} aria-hidden
                                className={i < lives ? "text-red-500 fill-red-500" : "text-neutral-300 dark:text-neutral-600"} />
                        ))}
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
                    <span className="flex items-center gap-1 text-primary-600 dark:text-primary-400 font-semibold text-sm">
                        <Flame size={16} aria-hidden />{t("comboLabel", { count: combo })}
                    </span>
                )}
                <span className="text-sm font-semibold">{runExp} EXP</span>
            </div>
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
