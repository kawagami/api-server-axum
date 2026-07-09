"use client";

import { answerVocabRun, finishVocabRun, getVocabMe, getVocabMistakes, startVocabRun } from "@/api/vocab";
import type { VocabMe, VocabMistake, VocabQuestion, VocabRunMode, VocabRunResult } from "@/types";
import { BookOpenCheck, CheckCircle2, Clock, Flame, GraduationCap, Heart, Loader2, Sparkles, Trophy } from "lucide-react";
import { useTranslations } from "next-intl";
import { useEffect, useRef, useState } from "react";

type Phase = "idle" | "playing" | "finished";

interface Feedback {
    correct: boolean;
    selectedIndex: number | null;
    correctChoiceIndex: number | null;
    correctText: string | null;
    gainedExp: number;
}

const FEEDBACK_MS = 1400;
const DURATIONS = [3, 5, 10] as const;

function hasLives(mode: VocabRunMode) {
    return mode === "survival" || mode === "timed_survival";
}
function hasTimer(mode: VocabRunMode) {
    return mode === "timed" || mode === "timed_survival";
}
function isReviewable(m: VocabMistake) {
    return m.wrong_count > m.correct_count;
}

export default function VocabClient({ initialMe, initialMistakes }: {
    initialMe: VocabMe; initialMistakes: VocabMistake[];
}) {
    const t = useTranslations("Vocab");
    const [me, setMe] = useState<VocabMe>(initialMe);
    const [mistakes, setMistakes] = useState<VocabMistake[]>(initialMistakes);
    const [phase, setPhase] = useState<Phase>("idle");
    const [mode, setMode] = useState<VocabRunMode>("survival");
    const [durationMin, setDurationMin] = useState<number>(10);
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState(false);

    const [runId, setRunId] = useState("");
    const [lives, setLives] = useState(3);
    const [combo, setCombo] = useState(0);
    const [runExp, setRunExp] = useState(0);
    const [total, setTotal] = useState(0);
    const [remaining, setRemaining] = useState(0);
    const [question, setQuestion] = useState<VocabQuestion | null>(null);
    const [feedback, setFeedback] = useState<Feedback | null>(null);
    const [result, setResult] = useState<VocabRunResult | null>(null);
    const [spellInput, setSpellInput] = useState("");

    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const inputRef = useRef<HTMLInputElement>(null);
    const runIdRef = useRef("");
    const deadlineRef = useRef<number | null>(null);
    const endedRef = useRef(false);
    useEffect(() => () => { if (timerRef.current) clearTimeout(timerRef.current); }, []);

    function refreshAfterRun() {
        getVocabMe().then(setMe).catch(() => { });
        getVocabMistakes().then(setMistakes).catch(() => { });
    }

    async function timeUp() {
        if (endedRef.current) return;
        endedRef.current = true;
        if (timerRef.current) clearTimeout(timerRef.current);
        try {
            const res = await finishVocabRun(runIdRef.current);
            if (res.result) {
                setFeedback(null);
                setResult(res.result);
                setPhase("finished");
                refreshAfterRun();
            }
        } catch {
            setError(true);
        }
    }

    // 進拼字題自動聚焦輸入框
    useEffect(() => {
        if (phase === "playing" && question?.kind === "spelling" && !feedback) {
            inputRef.current?.focus();
        }
    }, [phase, question, feedback]);

    // 限時模式:本地倒數,歸零呼叫 finish 結算
    useEffect(() => {
        if (phase !== "playing" || !hasTimer(mode)) return;
        const deadline = deadlineRef.current;
        if (!deadline) return;
        const tick = () => {
            const rem = Math.max(0, Math.ceil((deadline - Date.now()) / 1000));
            setRemaining(rem);
            if (rem <= 0) {
                clearInterval(iv);
                void timeUp();
            }
        };
        tick();
        const iv = setInterval(tick, 500);
        return () => clearInterval(iv);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [phase, mode]);

    async function start(runMode: VocabRunMode) {
        setBusy(true);
        setError(false);
        try {
            const res = await startVocabRun(runMode, hasTimer(runMode) ? durationMin : undefined);
            endedRef.current = false;
            runIdRef.current = res.run_id;
            deadlineRef.current = res.remaining_secs != null ? Date.now() + res.remaining_secs * 1000 : null;
            setMode(res.mode);
            setRunId(res.run_id);
            setLives(res.lives);
            setTotal(res.total ?? 0);
            setRemaining(res.remaining_secs ?? 0);
            setCombo(0);
            setRunExp(0);
            setQuestion(res.question);
            setFeedback(null);
            setResult(null);
            setSpellInput("");
            setPhase("playing");
        } catch {
            setError(true);
        } finally {
            setBusy(false);
        }
    }

    async function submit(input: { choice_index?: number; text?: string }) {
        if (busy || feedback || endedRef.current) return;
        setBusy(true);
        setError(false);
        try {
            const res = await answerVocabRun(runId, input);
            if (res.finished) endedRef.current = true; // 立即封鎖倒數,避免重複結算
            setLives(res.lives);
            setCombo(res.combo);
            setRunExp(res.run_exp);
            setFeedback({
                correct: res.correct,
                selectedIndex: input.choice_index ?? null,
                correctChoiceIndex: res.correct_choice_index ?? null,
                correctText: res.correct_text ?? null,
                gainedExp: res.gained_exp,
            });
            timerRef.current = setTimeout(() => {
                if (endedRef.current) return; // 已被倒數結束
                setFeedback(null);
                setSpellInput("");
                if (res.finished && res.result) {
                    endedRef.current = true;
                    setResult(res.result);
                    setPhase("finished");
                    refreshAfterRun();
                } else if (res.question) {
                    setQuestion(res.question);
                }
            }, FEEDBACK_MS);
        } catch {
            setError(true);
        } finally {
            setBusy(false);
        }
    }

    const reviewableCount = mistakes.filter(isReviewable).length;
    const bestOf = (m: VocabRunMode) => me.bests.find(b => b.mode === m);

    if (phase === "playing" && question) {
        return (
            <div className="flex flex-col gap-6">
                {mode === "review"
                    ? <ReviewHeader number={question.number} total={total} t={t} />
                    : <PlayHeader mode={mode} lives={lives} combo={combo} runExp={runExp}
                        number={question.number} remaining={remaining} t={t} />}
                {question.kind === "choice" ? (
                    <ChoiceCard question={question} feedback={feedback} busy={busy} t={t}
                        onPick={(i) => submit({ choice_index: i })} />
                ) : (
                    <SpellingCard question={question} feedback={feedback} busy={busy} t={t}
                        value={spellInput} onChange={setSpellInput} inputRef={inputRef}
                        onSubmit={() => { if (spellInput.trim()) submit({ text: spellInput }); }} />
                )}
                {error && <ErrorNote t={t} />}
            </div>
        );
    }

    if (phase === "finished" && result) {
        return (
            <div className="flex flex-col gap-6">
                <PageTitle t={t} />
                {mode === "review"
                    ? <ReviewResultCard result={result} busy={busy} onAgain={() => start("survival")} t={t} />
                    : <ScoredResultCard mode={mode} result={result} busy={busy} onAgain={() => start(mode)} t={t} />}
                <LevelCard me={me} t={t} />
                <MistakeBook mistakes={mistakes} t={t} />
            </div>
        );
    }

    // idle:入口畫面
    return (
        <div className="flex flex-col gap-6">
            <PageTitle t={t} />
            <LevelCard me={me} t={t} />
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col gap-5">
                <div className="grid grid-cols-2 gap-3">
                    <ModeButton label={t("modeSurvival")} desc={t("modeSurvivalDesc")}
                        best={bestOf("survival")?.correct_count} busy={busy}
                        onClick={() => start("survival")} t={t} />
                    <ModeButton label={t("modeTimedSurvival")} desc={t("modeTimedSurvivalDesc")}
                        best={bestOf("timed_survival")?.correct_count} busy={busy}
                        onClick={() => start("timed_survival")} t={t} />
                    <ModeButton label={t("modeTimed")} desc={t("modeTimedDesc")}
                        best={bestOf("timed")?.correct_count} busy={busy}
                        onClick={() => start("timed")} t={t} />
                    <button
                        onClick={() => start("review")}
                        disabled={busy || reviewableCount === 0}
                        title={reviewableCount === 0 ? t("noReview") : undefined}
                        className="flex flex-col items-center justify-center gap-1 px-4 py-4 rounded-lg border border-primary-500 text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950 font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    >
                        <span className="flex items-center gap-1"><BookOpenCheck size={18} />{t("reviewMistakes", { count: reviewableCount })}</span>
                        <span className="text-xs font-normal text-neutral-400 dark:text-neutral-500">{t("reviewMode")}</span>
                    </button>
                </div>

                {/* 限時模式共用的時長選擇 */}
                <div className="flex items-center justify-center gap-2 text-sm">
                    <Clock size={16} className="text-neutral-400" />
                    <span className="text-neutral-500 dark:text-neutral-400">{t("timeLimit")}</span>
                    {DURATIONS.map(d => (
                        <button
                            key={d}
                            onClick={() => setDurationMin(d)}
                            className={`px-3 py-1 rounded-full border transition-colors ${durationMin === d
                                ? "border-primary-500 bg-primary-500 text-white"
                                : "border-neutral-200 dark:border-neutral-600 hover:border-primary-400"}`}
                        >
                            {t("minutes", { n: d })}
                        </button>
                    ))}
                </div>

                {error && <ErrorNote t={t} />}
            </div>
            <MistakeBook mistakes={mistakes} t={t} />
        </div>
    );
}

type T = ReturnType<typeof useTranslations<"Vocab">>;

function fmtTime(secs: number) {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
}

function ModeButton({ label, desc, best, busy, onClick, t }: {
    label: string; desc: string; best?: number; busy: boolean; onClick: () => void; t: T;
}) {
    return (
        <button
            onClick={onClick}
            disabled={busy}
            className="flex flex-col items-center justify-center gap-1 px-4 py-4 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50"
        >
            <span>{label}</span>
            <span className="text-xs font-normal text-primary-100">{desc}</span>
            {best != null && <span className="text-xs font-normal text-primary-100">{t("bestShort", { n: best })}</span>}
        </button>
    );
}

function PageTitle({ t }: { t: T }) {
    return (
        <div className="flex items-center gap-2">
            <GraduationCap size={28} className="text-primary-500" />
            <div>
                <h1 className="text-2xl font-bold">{t("title")}</h1>
                <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("subtitle")}</p>
            </div>
        </div>
    );
}

function LevelCard({ me, t }: { me: VocabMe; t: T }) {
    const span = me.next_level_exp - me.level_exp;
    const progress = span > 0 ? Math.min(100, ((me.exp - me.level_exp) / span) * 100) : 100;
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col gap-3">
            <div className="flex items-end justify-between">
                <span className="text-lg font-bold text-primary-600 dark:text-primary-400">
                    {t("levelBadge", { level: me.level })}
                </span>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">
                    {me.exp} / {me.next_level_exp} EXP
                </span>
            </div>
            <div className="h-3 rounded-full bg-neutral-100 dark:bg-neutral-700 overflow-hidden">
                <div className="h-full rounded-full bg-primary-500 transition-all" style={{ width: `${progress}%` }} />
            </div>
        </div>
    );
}

function PlayHeader({ mode, lives, combo, runExp, number, remaining, t }: {
    mode: VocabRunMode; lives: number; combo: number; runExp: number; number: number; remaining: number; t: T;
}) {
    return (
        <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-3">
                {hasLives(mode) && (
                    <div className="flex gap-1">
                        {[0, 1, 2].map(i => (
                            <Heart key={i} size={20}
                                className={i < lives ? "text-red-500 fill-red-500" : "text-neutral-300 dark:text-neutral-600"} />
                        ))}
                    </div>
                )}
                {hasTimer(mode) && (
                    <span className={`flex items-center gap-1 font-mono font-semibold text-sm ${remaining <= 30 ? "text-red-500" : "text-neutral-600 dark:text-neutral-300"}`}>
                        <Clock size={16} />{fmtTime(remaining)}
                    </span>
                )}
            </div>
            <span className="text-sm font-medium text-neutral-500 dark:text-neutral-400">
                {t("questionNumber", { number })}
            </span>
            <div className="flex items-center gap-3">
                {combo > 1 && (
                    <span className="flex items-center gap-1 text-primary-600 dark:text-primary-400 font-semibold text-sm">
                        <Flame size={16} />{t("comboLabel", { count: combo })}
                    </span>
                )}
                <span className="text-sm font-semibold">{runExp} EXP</span>
            </div>
        </div>
    );
}

function ReviewHeader({ number, total, t }: { number: number; total: number; t: T }) {
    const progress = total > 0 ? Math.min(100, ((number - 1) / total) * 100) : 0;
    return (
        <div className="flex flex-col gap-2">
            <div className="flex items-center justify-between">
                <span className="flex items-center gap-1 text-primary-600 dark:text-primary-400 font-semibold text-sm">
                    <BookOpenCheck size={16} />{t("reviewMode")}
                </span>
                <span className="text-sm font-medium text-neutral-500 dark:text-neutral-400">{number} / {total}</span>
            </div>
            <div className="h-2 rounded-full bg-neutral-100 dark:bg-neutral-700 overflow-hidden">
                <div className="h-full rounded-full bg-primary-500 transition-all" style={{ width: `${progress}%` }} />
            </div>
        </div>
    );
}

function DifficultyDots({ difficulty }: { difficulty: number }) {
    return (
        <div className="flex gap-1" aria-hidden>
            {[1, 2, 3, 4, 5].map(i => (
                <span key={i} className={`w-1.5 h-1.5 rounded-full ${i <= difficulty ? "bg-primary-400" : "bg-neutral-200 dark:bg-neutral-600"}`} />
            ))}
        </div>
    );
}

function FeedbackBanner({ feedback, t }: { feedback: Feedback; t: T }) {
    return (
        <div className={`text-center font-semibold ${feedback.correct ? "text-green-600 dark:text-green-400" : "text-red-500"}`}>
            {feedback.correct
                ? <>{t("correct")}{feedback.gainedExp > 0 && <span className="ml-1">+{feedback.gainedExp} EXP</span>}</>
                : t("wrong")}
        </div>
    );
}

function ChoiceCard({ question, feedback, busy, onPick, t }: {
    question: VocabQuestion; feedback: Feedback | null; busy: boolean; onPick: (i: number) => void; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col gap-5">
            <div className="flex flex-col items-center gap-2">
                <DifficultyDots difficulty={question.difficulty} />
                <span className="text-3xl font-bold tracking-wide">{question.word}</span>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">{question.part_of_speech}</span>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">{t("chooseMeaning")}</span>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {question.options?.map((opt, i) => {
                    let cls = "border-neutral-200 dark:border-neutral-600 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950";
                    if (feedback) {
                        if (i === feedback.correctChoiceIndex) cls = "border-green-500 bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300";
                        else if (i === feedback.selectedIndex) cls = "border-red-500 bg-red-50 dark:bg-red-950 text-red-600 dark:text-red-300";
                        else cls = "border-neutral-200 dark:border-neutral-600 opacity-60";
                    }
                    return (
                        <button key={i} onClick={() => onPick(i)} disabled={busy || !!feedback}
                            className={`px-4 py-3 rounded-lg border text-left transition-colors ${cls}`}>
                            {opt}
                        </button>
                    );
                })}
            </div>
            {feedback && <FeedbackBanner feedback={feedback} t={t} />}
        </div>
    );
}

function SpellingCard({ question, feedback, busy, value, onChange, onSubmit, inputRef, t }: {
    question: VocabQuestion; feedback: Feedback | null; busy: boolean;
    value: string; onChange: (v: string) => void; onSubmit: () => void;
    inputRef: React.RefObject<HTMLInputElement | null>; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col gap-5">
            <div className="flex flex-col items-center gap-2">
                <DifficultyDots difficulty={question.difficulty} />
                <span className="text-2xl font-bold">{question.meaning_zh}</span>
                <span className="text-sm text-neutral-500 dark:text-neutral-400">{question.part_of_speech}</span>
                {question.sentence_masked && (
                    <p className="text-neutral-600 dark:text-neutral-300 text-center font-mono text-sm bg-neutral-50 dark:bg-neutral-700/50 rounded-lg px-4 py-3 w-full">
                        {question.sentence_masked}
                    </p>
                )}
                <span className="text-xs text-neutral-400 dark:text-neutral-500">
                    {t("spellingHint", { letter: question.hint_first_letter ?? "?", length: question.hint_length ?? 0 })}
                </span>
            </div>
            <form className="flex gap-2" onSubmit={(e) => { e.preventDefault(); onSubmit(); }}>
                <input ref={inputRef} value={value} onChange={(e) => onChange(e.target.value)}
                    disabled={busy || !!feedback} placeholder={t("inputPlaceholder")}
                    autoComplete="off" autoCapitalize="off" spellCheck={false}
                    className="flex-1 px-4 py-2 rounded-lg border border-neutral-200 dark:border-neutral-600 bg-transparent focus:outline-none focus:border-primary-400 font-mono" />
                <button type="submit" disabled={busy || !!feedback || !value.trim()}
                    className="px-5 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50">
                    {t("submit")}
                </button>
            </form>
            {feedback && (
                <div className="flex flex-col items-center gap-1">
                    <FeedbackBanner feedback={feedback} t={t} />
                    {!feedback.correct && feedback.correctText && (
                        <span className="text-sm text-neutral-500 dark:text-neutral-400">
                            {t("correctAnswerIs", { answer: feedback.correctText })}
                        </span>
                    )}
                </div>
            )}
        </div>
    );
}

function ScoredResultCard({ mode, result, busy, onAgain, t }: {
    mode: VocabRunMode; result: VocabRunResult; busy: boolean; onAgain: () => void; t: T;
}) {
    const overKey = mode === "timed" || mode === "timed_survival" ? "timeUpOver" : "runOver";
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col items-center gap-4">
            <Trophy size={40} className="text-primary-500" />
            <h2 className="text-xl font-bold">{t(overKey)}</h2>
            {result.new_best && (
                <span className="px-3 py-1 rounded-full bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300 text-sm font-semibold">
                    {t("newBest")}
                </span>
            )}
            <div className="grid grid-cols-3 gap-4 w-full text-center">
                <Stat label={t("answeredLabel")} value={result.answered_count} />
                <Stat label={t("correctLabel")} value={result.correct_count} />
                <Stat label={t("maxComboLabel")} value={result.max_combo} />
            </div>
            <div className="flex flex-col items-center gap-1">
                <span className="text-sm text-neutral-500 dark:text-neutral-400">{t("expGained")}</span>
                <span className="text-3xl font-bold text-primary-600 dark:text-primary-400">+{result.exp_gained}</span>
                {result.leveled_up && (
                    <span className="flex items-center gap-1 text-primary-600 dark:text-primary-300 font-semibold">
                        <Sparkles size={16} />{t("levelUp", { level: result.level })}
                    </span>
                )}
            </div>
            <AgainButton busy={busy} onAgain={onAgain} label={t("playAgain")} />
        </div>
    );
}

function ReviewResultCard({ result, busy, onAgain, t }: {
    result: VocabRunResult; busy: boolean; onAgain: () => void; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col items-center gap-4">
            <BookOpenCheck size={40} className="text-primary-500" />
            <h2 className="text-xl font-bold">{t("reviewOver")}</h2>
            <div className="grid grid-cols-3 gap-4 w-full text-center">
                <Stat label={t("reviewedLabel")} value={result.answered_count} />
                <Stat label={t("correctLabel")} value={result.correct_count} />
                <Stat label={t("graduatedLabel")} value={result.graduated ?? 0} />
            </div>
            <p className="text-sm text-neutral-500 dark:text-neutral-400 text-center">{t("reviewHint")}</p>
            <AgainButton busy={busy} onAgain={onAgain} label={t("start")} />
        </div>
    );
}

function AgainButton({ busy, onAgain, label }: { busy: boolean; onAgain: () => void; label: string }) {
    return (
        <button onClick={onAgain} disabled={busy}
            className="mt-2 px-6 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50 flex items-center gap-2">
            {busy ? <Loader2 size={18} className="animate-spin" /> : label}
        </button>
    );
}

function MistakeBook({ mistakes, t }: { mistakes: VocabMistake[]; t: T }) {
    if (mistakes.length === 0) {
        return (
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow text-center text-sm text-neutral-500 dark:text-neutral-400">
                {t("mistakeEmpty")}
            </div>
        );
    }
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow flex flex-col gap-2">
            <div className="flex items-center justify-between px-2">
                <h2 className="font-bold flex items-center gap-1">
                    <BookOpenCheck size={18} className="text-primary-500" />{t("mistakeBook")}
                </h2>
                <span className="text-xs text-neutral-400 dark:text-neutral-500">{t("mistakeCount", { count: mistakes.length })}</span>
            </div>
            <div className="max-h-80 overflow-auto flex flex-col divide-y divide-neutral-100 dark:divide-neutral-700">
                {mistakes.map(m => {
                    const mastered = m.correct_count >= m.wrong_count;
                    return (
                        <div key={m.word} className="flex items-center gap-3 py-2 px-2">
                            <div className="flex-1 min-w-0">
                                <div className="flex items-center gap-2">
                                    <span className="font-semibold truncate">{m.word}</span>
                                    <span className="text-xs text-neutral-400 dark:text-neutral-500 shrink-0">{m.part_of_speech}</span>
                                </div>
                                <div className="text-sm text-neutral-500 dark:text-neutral-400 truncate">{m.meaning_zh}</div>
                            </div>
                            <div className="flex items-center gap-2 shrink-0">
                                <span className="text-xs text-red-500" title={t("wrongCount")}>✗{m.wrong_count}</span>
                                <span className="text-xs text-green-600 dark:text-green-400" title={t("correctCount")}>✓{m.correct_count}</span>
                                {mastered && <CheckCircle2 size={16} className="text-green-500" aria-label={t("mastered")} />}
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

function Stat({ label, value }: { label: string; value: number }) {
    return (
        <div className="flex flex-col gap-1">
            <span className="text-2xl font-bold">{value}</span>
            <span className="text-xs text-neutral-500 dark:text-neutral-400">{label}</span>
        </div>
    );
}

function ErrorNote({ t }: { t: T }) {
    return <p className="text-sm text-red-500 text-center">{t("requestError")}</p>;
}
