"use client";

import { answerVocabRun, finishVocabRun, getVocabLeaderboard, getVocabMe, getVocabMistakes, startVocabRun } from "@/api/vocab";
import type { VocabLanguage, VocabLeaderboard, VocabLeaderboardPeriod, VocabMe, VocabMistakesPage, VocabQuestion, VocabRunMode, VocabRunResult } from "@/types";
import { Link } from "@/i18n/navigation";
import { BookOpenCheck, Clock } from "lucide-react";
import { useLocale, useTranslations } from "next-intl";
import { useCallback, useEffect, useRef, useState } from "react";
import { vocabSound } from "./sound";
import { canSpeak, speak } from "./speak";
import { addGuestRun, loadGuestStats, loadPrefs, savePrefs, type GuestStats } from "./prefs";
import { DURATIONS, FEEDBACK_MS, MISTAKE_PAGE_SIZE, SEARCH_DEBOUNCE_MS } from "./config";
import { LeaderboardCard } from "./leaderboard-card";
import { MistakeBook } from "./mistake-book";
import { DEFAULT_MISTAKE_QUERY, type Feedback, type MistakeQuery, type Pending, type Phase, hasTimer } from "./model";
import { ChoiceCard, PlayHeader, ReviewHeader, SpellingCard } from "./play-cards";
import { ReviewResultCard, ScoredResultCard } from "./result-cards";
import { ErrorNote, GuestBanner, GuestStatsCard, LevelCard, ModeButton, MuteButton, VocabHeading } from "./ui";

export default function VocabClient({ initialMe, initialMistakes, initialLeaderboard, isMember, language = "en" }: {
    initialMe: VocabMe | null; initialMistakes: VocabMistakesPage | null;
    initialLeaderboard: VocabLeaderboard | null; isMember: boolean; language?: VocabLanguage;
}) {
    const t = useTranslations("Vocab");
    const locale = useLocale();
    const ja = language === "ja";
    const pagePath = ja ? "/vocab-ja" : "/vocab";
    const loginHref = `/login?redirect=${encodeURIComponent(`/${locale}${pagePath}`)}`;
    const [me, setMe] = useState<VocabMe | null>(initialMe);
    const [board, setBoard] = useState<VocabLeaderboard | null>(initialLeaderboard);
    const [boardPeriod, setBoardPeriod] = useState<VocabLeaderboardPeriod>("weekly");
    const [boardLoading, setBoardLoading] = useState(false);
    const [boardError, setBoardError] = useState(false);
    const [phase, setPhase] = useState<Phase>("idle");
    const [mode, setMode] = useState<VocabRunMode>("survival");
    const [durationMin, setDurationMin] = useState<number>(10);
    const [lastMode, setLastMode] = useState<VocabRunMode | null>(null);
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState(false);
    const [canTts, setCanTts] = useState(false);
    const [guest, setGuest] = useState<GuestStats | null>(null);

    // 錯題本(後端強制分頁,搜尋/排序/篩選都在伺服器端做)
    const [mistakes, setMistakes] = useState<VocabMistakesPage | null>(initialMistakes);
    const [mistakeQuery, setMistakeQuery] = useState<MistakeQuery>(DEFAULT_MISTAKE_QUERY);
    const [searchInput, setSearchInput] = useState("");
    const [mistakeLoading, setMistakeLoading] = useState(false);
    const [mistakeError, setMistakeError] = useState(false);

    const [lives, setLives] = useState(3);
    const [combo, setCombo] = useState(0);
    const [runExp, setRunExp] = useState(0);
    const [total, setTotal] = useState(0);
    const [remaining, setRemaining] = useState(0);
    const [question, setQuestion] = useState<VocabQuestion | null>(null);
    const [feedback, setFeedback] = useState<Feedback | null>(null);
    const [result, setResult] = useState<VocabRunResult | null>(null);
    const [spellInput, setSpellInput] = useState("");
    const [muted, setMutedState] = useState(false);

    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const inputRef = useRef<HTMLInputElement>(null);
    const runIdRef = useRef("");
    const deadlineRef = useRef<number | null>(null);
    const endedRef = useRef(false);
    const pendingRef = useRef<Pending | null>(null);
    const mistakeSeqRef = useRef(0);
    // 日文拼字:羅馬字即打即轉假名(wanakana 只在日文版載入);IME 組字中不轉、不送出
    const toKanaRef = useRef<((s: string, opt?: object) => string) | null>(null);
    const composingRef = useRef(false);
    useEffect(() => {
        if (ja) import("wanakana").then(m => { toKanaRef.current = m.toKana; }).catch(() => { });
    }, [ja]);
    useEffect(() => () => { if (timerRef.current) clearTimeout(timerRef.current); }, []);
    // localStorage / speechSynthesis 都是 client-only:SSR 讀不到,只能 mount 後補同步,
    // 否則首次 render 的 HTML 會和 client 不一致(hydration mismatch)。
    /* eslint-disable react-hooks/set-state-in-effect */
    useEffect(() => {
        setMutedState(vocabSound.isMuted());
        setCanTts(canSpeak());
        const prefs = loadPrefs(10);
        setDurationMin(prefs.duration);
        setLastMode(prefs.lastMode);
        if (!isMember) setGuest(loadGuestStats(language));
    }, [isMember, language]);
    /* eslint-enable react-hooks/set-state-in-effect */

    function toggleMute() {
        const next = !muted;
        setMutedState(next);
        vocabSound.setMuted(next);
    }

    function pickDuration(d: number) {
        setDurationMin(d);
        savePrefs({ duration: d, lastMode });
    }

    const say = useCallback((text: string | null | undefined) => {
        if (text) speak(text, language);
    }, [language]);

    /**
     * 依當前搜尋條件抓錯題本;offset 0 為重抓,其他為「載入更多」。
     *
     * 帶請求序號閘:fetcher 是 Server Action、沒得 abort,只能在回應端裁決 ——
     * 否則「載入更多」慢回時會把之後換過條件的結果接在後面,拼出一份混合清單。
     */
    const fetchMistakes = useCallback((query: MistakeQuery, offset: number) => {
        if (!isMember) return;
        const seq = ++mistakeSeqRef.current;
        setMistakeLoading(true);
        setMistakeError(false);
        getVocabMistakes({ language, ...query, limit: MISTAKE_PAGE_SIZE, offset })
            .then(page => {
                if (seq !== mistakeSeqRef.current) return;
                setMistakes(prev => (offset > 0 && prev
                    ? { ...page, items: [...prev.items, ...page.items] }
                    : page));
            })
            .catch(() => { if (seq === mistakeSeqRef.current) setMistakeError(true); })
            .finally(() => { if (seq === mistakeSeqRef.current) setMistakeLoading(false); });
    }, [isMember, language]);

    // 搜尋輸入 debounce;條件一變就從第一頁重抓
    useEffect(() => {
        if (!isMember) return;
        const trimmed = searchInput.trim();
        if (trimmed === mistakeQuery.q) return;
        const id = setTimeout(() => setMistakeQuery(q => ({ ...q, q: trimmed })), SEARCH_DEBOUNCE_MS);
        return () => clearTimeout(id);
    }, [searchInput, mistakeQuery.q, isMember]);

    // 首次 render 用 SSR 的資料,不重抓;之後條件變才打
    const firstQueryRef = useRef(true);
    useEffect(() => {
        if (firstQueryRef.current) {
            firstQueryRef.current = false;
            return;
        }
        fetchMistakes(mistakeQuery, 0);
    }, [mistakeQuery, fetchMistakes]);

    function refreshAfterRun() {
        if (!isMember) return; // 訪客不打會員端點(會 401 轉登入);訪客局也不落地、榜不會變
        getVocabMe(language).then(setMe).catch(() => { });
        fetchMistakes(mistakeQuery, 0);
        getVocabLeaderboard(language, boardPeriod).then(setBoard).catch(() => { });
    }

    function switchBoardPeriod(period: VocabLeaderboardPeriod) {
        if (period === boardPeriod || boardLoading) return;
        setBoardPeriod(period);
        setBoardLoading(true);
        setBoardError(false);
        getVocabLeaderboard(language, period)
            .then(setBoard)
            .catch(() => setBoardError(true))
            .finally(() => setBoardLoading(false));
    }

    function handleSpellChange(v: string) {
        if (ja && toKanaRef.current && !composingRef.current) {
            setSpellInput(toKanaRef.current(v, { IMEMode: true }));
        } else {
            setSpellInput(v);
        }
    }

    /** 一局結束的共用收尾:訪客只累加本機紀錄,會員重抓伺服器資料 */
    function settle(runResult: VocabRunResult) {
        setResult(runResult);
        setPhase("finished");
        if (isMember) {
            refreshAfterRun();
        } else if (mode !== "review") {
            setGuest(addGuestRun(language, {
                correctCount: runResult.correct_count,
                expGained: runResult.exp_gained,
            }));
        }
    }

    /** 套用被回饋畫面攔住的下一步(倒數到了或使用者主動跳過) */
    const advance = useCallback(() => {
        const next = pendingRef.current;
        if (!next) return;
        pendingRef.current = null;
        if (timerRef.current) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
        setFeedback(null);
        setSpellInput("");
        if (next.finished && next.result) {
            // 本題結束對局:一定顯示結算(endedRef 已設,不可用來擋這裡)
            if (next.leveledUp) vocabSound.levelUp();
            settle(next.result);
        } else if (!endedRef.current && next.question) {
            // 未結束才換下一題;若期間被倒數結束則不動,交給 timeUp 的結算
            setQuestion(next.question);
        }
        // settle 依賴一票 state setter 與 mode,拆出來只會讓 deps 更長
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [isMember, language, mode, mistakeQuery, boardPeriod]);

    async function timeUp() {
        if (endedRef.current) return;
        endedRef.current = true;
        if (timerRef.current) clearTimeout(timerRef.current);
        pendingRef.current = null;
        vocabSound.timeUp();
        try {
            const res = await finishVocabRun(runIdRef.current);
            if (res.result) {
                setFeedback(null);
                settle(res.result);
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
        vocabSound.warmup(); // 使用者手勢內解 autoplay 鎖
        setBusy(true);
        setError(false);
        try {
            const res = await startVocabRun(runMode, hasTimer(runMode) ? durationMin : undefined, language);
            endedRef.current = false;
            pendingRef.current = null;
            runIdRef.current = res.run_id;
            deadlineRef.current = res.remaining_secs != null ? Date.now() + res.remaining_secs * 1000 : null;
            setMode(res.mode);
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
            setLastMode(res.mode);
            savePrefs({ duration: durationMin, lastMode: res.mode });
        } catch {
            setError(true);
        } finally {
            setBusy(false);
        }
    }

    /** 放棄這一局回入口(送不出答案時的唯一出路) */
    function abort() {
        if (timerRef.current) clearTimeout(timerRef.current);
        timerRef.current = null;
        pendingRef.current = null;
        endedRef.current = true;
        setFeedback(null);
        setQuestion(null);
        setResult(null);
        setError(false);
        setPhase("idle");
    }

    async function submit(input: { choice_index?: number; text?: string }) {
        if (busy || feedback || endedRef.current) return;
        setBusy(true);
        setError(false);
        try {
            const res = await answerVocabRun(runIdRef.current, input);
            if (res.finished) endedRef.current = true; // 立即封鎖倒數,避免重複結算
            if (res.correct) vocabSound.correct(); else vocabSound.wrong();
            setLives(res.lives);
            setCombo(res.combo);
            setRunExp(res.run_exp);
            setFeedback({
                correct: res.correct,
                selectedIndex: input.choice_index ?? null,
                correctChoiceIndex: res.correct_choice_index ?? null,
                correctText: res.correct_text ?? null,
                reading: res.reading ?? null,
                gainedExp: res.gained_exp,
            });
            pendingRef.current = {
                finished: res.finished,
                leveledUp: res.result?.leveled_up ?? false,
                result: res.result ?? null,
                question: res.question ?? null,
            };
            timerRef.current = setTimeout(advance, FEEDBACK_MS);
        } catch {
            setError(true);
        } finally {
            setBusy(false);
        }
    }

    // 鍵盤操作:回饋中 Enter/Space 立刻續題,選擇題 1–4 直接作答
    useEffect(() => {
        if (phase !== "playing") return;
        const onKey = (e: KeyboardEvent) => {
            if (feedback) {
                if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    advance();
                }
                return;
            }
            if (busy || question?.kind !== "choice") return;
            const picked = Number(e.key);
            if (Number.isInteger(picked) && picked >= 1 && picked <= (question.options?.length ?? 0)) {
                e.preventDefault();
                void submit({ choice_index: picked - 1 });
            }
        };
        window.addEventListener("keydown", onKey);
        return () => window.removeEventListener("keydown", onKey);
        // submit 每次 render 都是新的 closure,但它讀的都是最新 state,不需要進 deps
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [phase, feedback, question, busy, advance]);

    const reviewableCount = mistakes?.reviewable ?? 0;
    const bestOf = (m: VocabRunMode) => me?.bests.find(b => b.mode === m);
    const mistakeBook = isMember && (
        <MistakeBook page={mistakes} query={mistakeQuery} searchInput={searchInput}
            loading={mistakeLoading} error={mistakeError} canTts={canTts} ja={ja}
            onSearch={setSearchInput}
            onQuery={patch => setMistakeQuery(q => ({ ...q, ...patch }))}
            onMore={() => fetchMistakes(mistakeQuery, mistakes?.items.length ?? 0)}
            onSpeak={say} t={t} />
    );
    const leaderboardCard = board && (
        <LeaderboardCard board={board} period={boardPeriod} loading={boardLoading} error={boardError}
            isMember={isMember} onPeriod={switchBoardPeriod} t={t} />
    );

    if (phase === "playing" && question) {
        const fxClass = feedback ? (feedback.correct ? "fx-pop" : "fx-shake") : "";
        return (
            <div className="flex flex-col gap-6">
                <div className="flex items-center gap-3">
                    <div className="flex-1 min-w-0">
                        {mode === "review"
                            ? <ReviewHeader number={question.number} total={total} t={t} />
                            : <PlayHeader mode={mode} lives={lives} combo={combo} runExp={runExp}
                                number={question.number} remaining={remaining} t={t} />}
                    </div>
                    <MuteButton muted={muted} onToggle={toggleMute} t={t} />
                </div>
                {/* 回饋期間整塊可點 = 立刻續題 */}
                <div className={`relative ${fxClass}`} onClick={feedback ? advance : undefined}>
                    {question.kind === "choice" ? (
                        <ChoiceCard question={question} feedback={feedback} busy={busy} ja={ja} t={t}
                            canTts={canTts} onSpeak={say}
                            onPick={(i) => submit({ choice_index: i })} />
                    ) : (
                        <SpellingCard question={question} feedback={feedback} busy={busy} ja={ja} t={t}
                            canTts={canTts} onSpeak={say}
                            value={spellInput} onChange={handleSpellChange} inputRef={inputRef}
                            composingRef={composingRef}
                            onSubmit={() => { if (spellInput.trim()) submit({ text: spellInput }); }} />
                    )}
                    {feedback?.correct && feedback.gainedExp > 0 && (
                        <span className="fx-float pointer-events-none absolute left-1/2 -translate-x-1/2 top-1 text-primary-500 font-bold text-lg">
                            +{feedback.gainedExp} EXP
                        </span>
                    )}
                </div>
                {error && (
                    <div className="flex flex-col items-center gap-2">
                        <ErrorNote t={t} />
                        <button onClick={abort}
                            className="text-sm text-primary-600 dark:text-primary-400 hover:underline">
                            {t("backToMenu")}
                        </button>
                    </div>
                )}
            </div>
        );
    }

    if (phase === "finished" && result) {
        return (
            <div className="flex flex-col gap-6">
                <VocabHeading ja={ja} t={t} />
                {mode === "review"
                    ? <ReviewResultCard result={result} busy={busy} onAgain={() => start("review")}
                        onMenu={abort} t={t} />
                    : <ScoredResultCard mode={mode} result={result} busy={busy} isMember={isMember}
                        loginHref={loginHref} onAgain={() => start(mode)} onMenu={abort} t={t} />}
                {me && <LevelCard me={me} t={t} />}
                {!isMember && guest && <GuestStatsCard stats={guest} t={t} />}
                {leaderboardCard}
                {mistakeBook}
            </div>
        );
    }

    // idle:入口畫面
    const reviewDisabled = !isMember || reviewableCount === 0;
    return (
        <div className="flex flex-col gap-6">
            <div className="flex items-start justify-between gap-2">
                <VocabHeading ja={ja} t={t} />
                <Link href={ja ? "/vocab" : "/vocab-ja"}
                    className="shrink-0 text-sm text-primary-600 dark:text-primary-400 hover:underline mt-1">
                    {ja ? t("switchToEn") : t("switchToJa")}
                </Link>
            </div>
            {me ? <LevelCard me={me} t={t} /> : <GuestBanner loginHref={loginHref} t={t} />}
            {!isMember && guest && guest.runs > 0 && <GuestStatsCard stats={guest} t={t} />}
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex flex-col gap-5">
                <div className="grid grid-cols-2 gap-3">
                    <ModeButton label={t("modeSurvival")} desc={t("modeSurvivalDesc")}
                        best={bestOf("survival")?.correct_count} busy={busy} last={lastMode === "survival"}
                        onClick={() => start("survival")} t={t} />
                    <ModeButton label={t("modeTimedSurvival")} desc={t("modeTimedSurvivalDesc")}
                        best={bestOf("timed_survival")?.correct_count} busy={busy} last={lastMode === "timed_survival"}
                        onClick={() => start("timed_survival")} t={t} />
                    <ModeButton label={t("modeTimed")} desc={t("modeTimedDesc")}
                        best={bestOf("timed")?.correct_count} busy={busy} last={lastMode === "timed"}
                        onClick={() => start("timed")} t={t} />
                    <button
                        onClick={() => start("review")}
                        disabled={busy || reviewDisabled}
                        title={!isMember ? t("loginToReview") : reviewableCount === 0 ? t("noReview") : undefined}
                        className="flex flex-col items-center justify-center gap-1 px-4 py-4 rounded-lg border border-primary-500 text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950 font-semibold transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                    >
                        <span className="flex items-center gap-1"><BookOpenCheck size={18} />{t("reviewMistakes", { count: isMember ? reviewableCount : 0 })}</span>
                        <span className="text-xs font-normal text-neutral-400 dark:text-neutral-500">{isMember ? t("reviewMode") : t("memberOnly")}</span>
                    </button>
                </div>

                {/* 限時模式共用的時長選擇 */}
                <div className="flex items-center justify-center gap-2 text-sm">
                    <Clock size={16} className="text-neutral-400" />
                    <span className="text-neutral-500 dark:text-neutral-400">{t("timeLimit")}</span>
                    {DURATIONS.map(d => (
                        <button
                            key={d}
                            onClick={() => pickDuration(d)}
                            className={`px-3 py-1 rounded-full border transition-colors ${durationMin === d
                                ? "border-primary-500 bg-primary-500 text-white"
                                : "border-neutral-200 dark:border-neutral-600 hover:border-primary-400"}`}
                        >
                            {t("minutes", { n: d })}
                        </button>
                    ))}
                </div>

                <details className="text-sm text-neutral-500 dark:text-neutral-400">
                    <summary className="cursor-pointer font-medium text-neutral-600 dark:text-neutral-300">
                        {t("rulesToggle")}
                    </summary>
                    <p className="mt-2 leading-relaxed">{t("rules")}</p>
                    <p className="mt-1 leading-relaxed">{t("keyboardHint")}</p>
                </details>

                {error && <ErrorNote t={t} />}
            </div>
            {leaderboardCard}
            {mistakeBook}
        </div>
    );
}
