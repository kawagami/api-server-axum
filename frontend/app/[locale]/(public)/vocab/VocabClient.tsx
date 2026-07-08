"use client";

import { answerVocabRun, getVocabMe, startVocabRun } from "@/api/vocab";
import type { VocabMe, VocabQuestion, VocabRunResult } from "@/types";
import { Flame, GraduationCap, Heart, Loader2, Sparkles, Trophy } from "lucide-react";
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

export default function VocabClient({ initialMe }: { initialMe: VocabMe }) {
    const t = useTranslations("Vocab");
    const [me, setMe] = useState<VocabMe>(initialMe);
    const [phase, setPhase] = useState<Phase>("idle");
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState(false);

    const [runId, setRunId] = useState("");
    const [lives, setLives] = useState(3);
    const [combo, setCombo] = useState(0);
    const [runExp, setRunExp] = useState(0);
    const [question, setQuestion] = useState<VocabQuestion | null>(null);
    const [feedback, setFeedback] = useState<Feedback | null>(null);
    const [result, setResult] = useState<VocabRunResult | null>(null);
    const [spellInput, setSpellInput] = useState("");

    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const inputRef = useRef<HTMLInputElement>(null);
    useEffect(() => () => { if (timerRef.current) clearTimeout(timerRef.current); }, []);

    // 進拼字題自動聚焦輸入框
    useEffect(() => {
        if (phase === "playing" && question?.kind === "spelling" && !feedback) {
            inputRef.current?.focus();
        }
    }, [phase, question, feedback]);

    async function start() {
        setBusy(true);
        setError(false);
        try {
            const res = await startVocabRun();
            setRunId(res.run_id);
            setLives(res.lives);
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
        if (busy || feedback) return;
        setBusy(true);
        setError(false);
        try {
            const res = await answerVocabRun(runId, input);
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
                setFeedback(null);
                setSpellInput("");
                if (res.finished && res.result) {
                    setResult(res.result);
                    setPhase("finished");
                    // 重新抓最新等級 / 最佳紀錄(結算已含 total_exp,這裡補 best/words_learned)
                    getVocabMe().then(setMe).catch(() => { });
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

    if (phase === "playing" && question) {
        return (
            <div className="flex flex-col gap-6">
                <GameHeader lives={lives} combo={combo} runExp={runExp} number={question.number} t={t} />
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
                <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col items-center gap-4">
                    <Trophy size={40} className="text-primary-500" />
                    <h2 className="text-xl font-bold">{t("runOver")}</h2>
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
                        <span className="text-3xl font-bold text-primary-600 dark:text-primary-400">
                            +{result.exp_gained}
                        </span>
                        {result.leveled_up && (
                            <span className="flex items-center gap-1 text-primary-600 dark:text-primary-300 font-semibold">
                                <Sparkles size={16} />
                                {t("levelUp", { level: result.level })}
                            </span>
                        )}
                    </div>
                    <button
                        onClick={start}
                        disabled={busy}
                        className="mt-2 px-6 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50"
                    >
                        {busy ? <Loader2 size={18} className="animate-spin" /> : t("playAgain")}
                    </button>
                    {error && <ErrorNote t={t} />}
                </div>
                <LevelCard me={me} t={t} />
            </div>
        );
    }

    // idle:入口畫面
    return (
        <div className="flex flex-col gap-6">
            <PageTitle t={t} />
            <LevelCard me={me} t={t} />
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col gap-4">
                <div className="grid grid-cols-3 gap-4 text-center">
                    <Stat label={t("totalRuns")} value={me.total_runs} />
                    <Stat label={t("wordsLearned")} value={me.words_learned} />
                    <Stat label={t("bestCorrect")} value={me.best?.correct_count ?? 0} />
                </div>
                <p className="text-sm text-neutral-500 dark:text-neutral-400 text-center">{t("rules")}</p>
                <button
                    onClick={start}
                    disabled={busy}
                    className="self-center px-8 py-3 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold text-lg transition-colors disabled:opacity-50 flex items-center gap-2"
                >
                    {busy ? <Loader2 size={20} className="animate-spin" /> : t("start")}
                </button>
                {error && <ErrorNote t={t} />}
            </div>
        </div>
    );
}

type T = ReturnType<typeof useTranslations<"Vocab">>;

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
                <div
                    className="h-full rounded-full bg-primary-500 transition-all"
                    style={{ width: `${progress}%` }}
                />
            </div>
        </div>
    );
}

function GameHeader({ lives, combo, runExp, number, t }: {
    lives: number; combo: number; runExp: number; number: number; t: T;
}) {
    return (
        <div className="flex items-center justify-between">
            <div className="flex gap-1">
                {[0, 1, 2].map(i => (
                    <Heart
                        key={i}
                        size={22}
                        className={i < lives
                            ? "text-red-500 fill-red-500"
                            : "text-neutral-300 dark:text-neutral-600"}
                    />
                ))}
            </div>
            <span className="text-sm font-medium text-neutral-500 dark:text-neutral-400">
                {t("questionNumber", { number })}
            </span>
            <div className="flex items-center gap-3">
                {combo > 1 && (
                    <span className="flex items-center gap-1 text-primary-600 dark:text-primary-400 font-semibold text-sm">
                        <Flame size={16} />
                        {t("comboLabel", { count: combo })}
                    </span>
                )}
                <span className="text-sm font-semibold">{runExp} EXP</span>
            </div>
        </div>
    );
}

function DifficultyDots({ difficulty }: { difficulty: number }) {
    return (
        <div className="flex gap-1" aria-hidden>
            {[1, 2, 3, 4, 5].map(i => (
                <span
                    key={i}
                    className={`w-1.5 h-1.5 rounded-full ${i <= difficulty ? "bg-primary-400" : "bg-neutral-200 dark:bg-neutral-600"}`}
                />
            ))}
        </div>
    );
}

function FeedbackBanner({ feedback, t }: { feedback: Feedback; t: T }) {
    return (
        <div className={`text-center font-semibold ${feedback.correct ? "text-green-600 dark:text-green-400" : "text-red-500"}`}>
            {feedback.correct
                ? <>{t("correct")} <span className="ml-1">+{feedback.gainedExp} EXP</span></>
                : t("wrong")}
        </div>
    );
}

function ChoiceCard({ question, feedback, busy, onPick, t }: {
    question: VocabQuestion; feedback: Feedback | null; busy: boolean;
    onPick: (i: number) => void; t: T;
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
                        if (i === feedback.correctChoiceIndex) {
                            cls = "border-green-500 bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300";
                        } else if (i === feedback.selectedIndex) {
                            cls = "border-red-500 bg-red-50 dark:bg-red-950 text-red-600 dark:text-red-300";
                        } else {
                            cls = "border-neutral-200 dark:border-neutral-600 opacity-60";
                        }
                    }
                    return (
                        <button
                            key={i}
                            onClick={() => onPick(i)}
                            disabled={busy || !!feedback}
                            className={`px-4 py-3 rounded-lg border text-left transition-colors ${cls}`}
                        >
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
                    {t("spellingHint", {
                        letter: question.hint_first_letter ?? "?",
                        length: question.hint_length ?? 0,
                    })}
                </span>
            </div>
            <form
                className="flex gap-2"
                onSubmit={(e) => { e.preventDefault(); onSubmit(); }}
            >
                <input
                    ref={inputRef}
                    value={value}
                    onChange={(e) => onChange(e.target.value)}
                    disabled={busy || !!feedback}
                    placeholder={t("inputPlaceholder")}
                    autoComplete="off"
                    autoCapitalize="off"
                    spellCheck={false}
                    className="flex-1 px-4 py-2 rounded-lg border border-neutral-200 dark:border-neutral-600 bg-transparent focus:outline-none focus:border-primary-400 font-mono"
                />
                <button
                    type="submit"
                    disabled={busy || !!feedback || !value.trim()}
                    className="px-5 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50"
                >
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
