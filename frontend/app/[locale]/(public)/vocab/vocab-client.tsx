"use client";

import { answerVocabRun, finishVocabRun, getVocabLeaderboard, getVocabMe, getVocabMistakes, startVocabRun } from "@/api/vocab";
import type { VocabLanguage, VocabLeaderboard, VocabLeaderboardPeriod, VocabLeaderboardRow, VocabMe, VocabMistake, VocabMistakeSort, VocabMistakesPage, VocabQuestion, VocabRunMode, VocabRunResult } from "@/types";
import { Link } from "@/i18n/navigation";
import { BookOpenCheck, Check, CheckCircle2, Clock, Flame, GraduationCap, Heart, Loader2, LogIn, Search, Sparkles, Trophy, Volume2, VolumeX, X } from "lucide-react";
import { useLocale, useTranslations } from "next-intl";
import { useCallback, useEffect, useRef, useState } from "react";
import { vocabSound } from "./sound";
import { canSpeak, speak } from "./speak";
import { addGuestRun, loadGuestStats, loadPrefs, savePrefs, type GuestStats } from "./prefs";
import { DURATIONS, FEEDBACK_MS, MISTAKE_PAGE_SIZE, SEARCH_DEBOUNCE_MS } from "./config";
import PageTitle from "@/components/page-title";

type Phase = "idle" | "playing" | "finished";

interface Feedback {
    correct: boolean;
    selectedIndex: number | null;
    correctChoiceIndex: number | null;
    correctText: string | null;
    reading: string | null; // 日文局答後回饋的讀音
    gainedExp: number;
}

/** 回饋期間先攔住的下一步;倒數到了或使用者主動跳過才套用 */
interface Pending {
    finished: boolean;
    leveledUp: boolean;
    result: VocabRunResult | null;
    question: VocabQuestion | null;
}

interface MistakeQuery {
    q: string;
    sort: VocabMistakeSort;
    unmastered: boolean;
}

const DEFAULT_MISTAKE_QUERY: MistakeQuery = { q: "", sort: "wrong", unmastered: false };

function hasLives(mode: VocabRunMode) {
    return mode === "survival" || mode === "timed_survival";
}
function hasTimer(mode: VocabRunMode) {
    return mode === "timed" || mode === "timed_survival";
}

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

type T = ReturnType<typeof useTranslations<"Vocab">>;

function fmtTime(secs: number) {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
}

/** 發音鈕;裝置沒有語音引擎就整顆不 render(不要留按了沒反應的鈕) */
function SpeakButton({ text, canTts, onSpeak, t, size = 16 }: {
    text: string | null | undefined; canTts: boolean;
    onSpeak: (text: string) => void; t: T; size?: number;
}) {
    if (!canTts || !text) return null;
    return (
        <button type="button" onClick={(e) => { e.stopPropagation(); onSpeak(text); }}
            aria-label={t("speak")} title={t("speak")}
            className="shrink-0 p-1 rounded text-neutral-400 hover:text-primary-500 transition-colors">
            <Volume2 size={size} />
        </button>
    );
}

function ModeButton({ label, desc, best, busy, last, onClick, t }: {
    label: string; desc: string; best?: number; busy: boolean; last: boolean; onClick: () => void; t: T;
}) {
    return (
        <button
            onClick={onClick}
            disabled={busy}
            className="relative flex flex-col items-center justify-center gap-1 px-4 py-4 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50"
        >
            {last && (
                <span className="absolute top-1 right-2 text-[10px] font-normal text-primary-100">
                    {t("lastPlayed")}
                </span>
            )}
            <span>{label}</span>
            <span className="text-xs font-normal text-primary-100">{desc}</span>
            {best != null && <span className="text-xs font-normal text-primary-100">{t("bestShort", { n: best })}</span>}
        </button>
    );
}

function MuteButton({ muted, onToggle, t }: { muted: boolean; onToggle: () => void; t: T }) {
    return (
        <button
            onClick={onToggle}
            aria-label={muted ? t("unmute") : t("mute")}
            title={muted ? t("unmute") : t("mute")}
            className="shrink-0 p-2 rounded-lg text-neutral-400 hover:text-primary-500 hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
        >
            {muted ? <VolumeX size={18} /> : <Volume2 size={18} />}
        </button>
    );
}

// 標題規格走全站共用的 PageTitle，只是把圖示塞進 title 裡
function VocabHeading({ ja, t }: { ja: boolean; t: T }) {
    return (
        <PageTitle
            title={
                <span className="flex items-center gap-2">
                    <GraduationCap size={26} className="text-primary-500" />
                    {t(ja ? "titleJa" : "title")}
                </span>
            }
            description={t(ja ? "subtitleJa" : "subtitle")}
        />
    );
}

function GuestBanner({ loginHref, t }: { loginHref: string; t: T }) {
    return (
        <div className="bg-primary-50 dark:bg-primary-950 border border-primary-200 dark:border-primary-800 rounded-xl p-4 flex items-center justify-between gap-3">
            <p className="text-sm text-primary-700 dark:text-primary-300">{t("guestBanner")}</p>
            <Link
                href={loginHref}
                className="shrink-0 flex items-center gap-1 px-4 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white text-sm font-semibold transition-colors"
            >
                <LogIn size={16} />{t("login")}
            </Link>
        </div>
    );
}

/** 訪客本機紀錄。訪客局在後端不落地,所以講清楚只存在這個瀏覽器,不會上榜。 */
function GuestStatsCard({ stats, t }: { stats: GuestStats; t: T }) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow-sm flex flex-col gap-2">
            <div className="flex items-baseline justify-between gap-2">
                <h2 className="font-bold text-sm">{t("guestLocalTitle")}</h2>
                <span className="text-xs text-neutral-400 dark:text-neutral-500">{t("guestLocalHint")}</span>
            </div>
            <div className="grid grid-cols-3 gap-2 text-center">
                <Stat label={t("guestLocalRuns")} value={stats.runs} />
                <Stat label={t("guestLocalBest")} value={stats.bestCorrect} />
                <Stat label={t("guestLocalExp")} value={stats.exp} />
            </div>
        </div>
    );
}

function LevelCard({ me, t }: { me: VocabMe; t: T }) {
    const span = me.next_level_exp - me.level_exp;
    const progress = span > 0 ? Math.min(100, ((me.exp - me.level_exp) / span) * 100) : 100;
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex flex-col gap-3">
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
            <div className="grid grid-cols-3 gap-2 text-center border-t border-neutral-100 dark:border-neutral-700 pt-3">
                <Stat label={t("streakLabel")} value={me.streak_days} suffix={t("daysUnit")} />
                <Stat label={t("wordsLearnedLabel")} value={me.words_learned} />
                <Stat label={t("totalRunsLabel")} value={me.total_runs} />
            </div>
            {me.streak_days > 0 && !me.played_today && (
                <p className="flex items-center justify-center gap-1 text-xs text-primary-600 dark:text-primary-400">
                    <Flame size={14} />{t("streakAtRisk")}
                </p>
            )}
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

function ReviewHeader({ number, total, t }: { number: number; total: number; t: T }) {
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

function ChoiceCard({ question, feedback, busy, ja, canTts, onPick, onSpeak, t }: {
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

function SpellingCard({ question, feedback, busy, ja, canTts, value, onChange, onSubmit, onSpeak, inputRef, composingRef, t }: {
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

function ScoredResultCard({ mode, result, busy, isMember, loginHref, onAgain, onMenu, t }: {
    mode: VocabRunMode; result: VocabRunResult; busy: boolean; isMember: boolean; loginHref: string;
    onAgain: () => void; onMenu: () => void; t: T;
}) {
    const overKey = mode === "timed" || mode === "timed_survival" ? "timeUpOver" : "runOver";
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex flex-col items-center gap-4">
            <Trophy size={40} className="text-primary-500" aria-hidden />
            <h2 className="text-xl font-bold">{t(overKey)}</h2>
            {isMember && result.new_best && (
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
                {isMember && result.leveled_up && (
                    <span className="flex items-center gap-1 text-primary-600 dark:text-primary-300 font-semibold">
                        <Sparkles size={16} aria-hidden />{t("levelUp", { level: result.level })}
                    </span>
                )}
            </div>
            {!isMember && (
                <Link
                    href={loginHref}
                    className="flex items-center gap-1 text-sm text-primary-600 dark:text-primary-400 hover:underline"
                >
                    <LogIn size={15} />{t("guestSavePrompt")}
                </Link>
            )}
            <ResultActions busy={busy} onAgain={onAgain} onMenu={onMenu} againLabel={t("playAgain")} t={t} />
        </div>
    );
}

function ReviewResultCard({ result, busy, onAgain, onMenu, t }: {
    result: VocabRunResult; busy: boolean; onAgain: () => void; onMenu: () => void; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex flex-col items-center gap-4">
            <BookOpenCheck size={40} className="text-primary-500" aria-hidden />
            <h2 className="text-xl font-bold">{t("reviewOver")}</h2>
            <div className="grid grid-cols-3 gap-4 w-full text-center">
                <Stat label={t("reviewedLabel")} value={result.answered_count} />
                <Stat label={t("correctLabel")} value={result.correct_count} />
                <Stat label={t("graduatedLabel")} value={result.graduated ?? 0} />
            </div>
            <p className="text-sm text-neutral-500 dark:text-neutral-400 text-center">{t("reviewHint")}</p>
            <ResultActions busy={busy} onAgain={onAgain} onMenu={onMenu} againLabel={t("reviewAgain")} t={t} />
        </div>
    );
}

function ResultActions({ busy, onAgain, onMenu, againLabel, t }: {
    busy: boolean; onAgain: () => void; onMenu: () => void; againLabel: string; t: T;
}) {
    return (
        <div className="mt-2 flex items-center gap-3">
            <button onClick={onAgain} disabled={busy}
                className="px-6 py-2 rounded-lg bg-primary-500 hover:bg-primary-600 text-white font-semibold transition-colors disabled:opacity-50 flex items-center gap-2">
                {busy ? <Loader2 size={18} className="animate-spin" /> : againLabel}
            </button>
            <button onClick={onMenu} disabled={busy}
                className="px-4 py-2 rounded-lg border border-neutral-200 dark:border-neutral-600 text-sm hover:border-primary-400 transition-colors disabled:opacity-50">
                {t("backToMenu")}
            </button>
        </div>
    );
}

const PERIODS: VocabLeaderboardPeriod[] = ["weekly", "monthly", "all"];
const PERIOD_KEYS: Record<VocabLeaderboardPeriod, "lbWeekly" | "lbMonthly" | "lbAll"> = {
    weekly: "lbWeekly", monthly: "lbMonthly", all: "lbAll",
};

/** OAuth 頭像是外部 URL,壞圖要退回首字母而不是留一個破圖示 */
function Avatar({ row }: { row: VocabLeaderboardRow }) {
    const [broken, setBroken] = useState(false);
    if (row.avatar_url && !broken) {
        return (
            // 外部 URL 無法經 next/image 最佳化
            // eslint-disable-next-line @next/next/no-img-element
            <img src={row.avatar_url} alt="" loading="lazy" referrerPolicy="no-referrer"
                onError={() => setBroken(true)}
                className="w-7 h-7 shrink-0 rounded-full object-cover" />
        );
    }
    return (
        <span className="w-7 h-7 shrink-0 rounded-full bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300 flex items-center justify-center text-xs font-semibold">
            {row.name.charAt(0)}
        </span>
    );
}

function LeaderboardCard({ board, period, loading, error, isMember, onPeriod, t }: {
    board: VocabLeaderboard; period: VocabLeaderboardPeriod; loading: boolean; error: boolean;
    isMember: boolean; onPeriod: (p: VocabLeaderboardPeriod) => void; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow-sm flex flex-col gap-2">
            <div className="flex items-center justify-between px-2">
                <h2 className="font-bold flex items-center gap-1">
                    <Trophy size={18} className="text-primary-500" aria-hidden />{t("leaderboard")}
                </h2>
                <div className="flex gap-1">
                    {PERIODS.map(p => (
                        <button key={p} onClick={() => onPeriod(p)} disabled={loading}
                            aria-pressed={period === p}
                            className={`px-3 py-1 rounded-full text-xs border transition-colors ${period === p
                                ? "border-primary-500 bg-primary-500 text-white"
                                : "border-neutral-200 dark:border-neutral-600 hover:border-primary-400"}`}>
                            {t(PERIOD_KEYS[p])}
                        </button>
                    ))}
                </div>
            </div>
            {loading ? (
                <div className="flex justify-center py-6">
                    <Loader2 size={20} className="animate-spin text-neutral-400" />
                </div>
            ) : error ? (
                <p className="text-center text-sm text-red-500 py-4">{t("lbError")}</p>
            ) : board.top.length === 0 ? (
                <p className="text-center text-sm text-neutral-500 dark:text-neutral-400 py-4">{t("lbEmpty")}</p>
            ) : (
                <div className="flex flex-col divide-y divide-neutral-100 dark:divide-neutral-700">
                    {board.top.map(row => (
                        <div key={`${row.rank}|${row.name}`} className="flex items-center gap-3 py-2 px-2">
                            <span className={`w-7 h-7 shrink-0 rounded-full flex items-center justify-center text-xs font-bold ${row.rank <= 3
                                ? "bg-primary-500 text-white"
                                : "bg-neutral-100 dark:bg-neutral-700 text-neutral-500 dark:text-neutral-400"}`}>
                                {row.rank}
                            </span>
                            <Avatar row={row} />
                            <span className="flex-1 min-w-0 truncate font-medium">{row.name}</span>
                            <span className="text-xs text-neutral-400 dark:text-neutral-500 shrink-0">{t("lbRuns", { count: row.runs })}</span>
                            <span className="text-sm font-semibold text-primary-600 dark:text-primary-400 shrink-0">{row.exp} EXP</span>
                        </div>
                    ))}
                </div>
            )}
            {!loading && !error && isMember && (
                <p className="text-center text-xs text-neutral-500 dark:text-neutral-400 border-t border-neutral-100 dark:border-neutral-700 pt-2">
                    {board.me
                        ? t("lbMyRank", { rank: board.me.rank, exp: board.me.exp })
                        : t("lbNotRanked")}
                </p>
            )}
        </div>
    );
}

const SORTS: VocabMistakeSort[] = ["wrong", "recent", "difficulty", "word"];
const SORT_KEYS: Record<VocabMistakeSort, "msWrong" | "msRecent" | "msDifficulty" | "msWord"> = {
    wrong: "msWrong", recent: "msRecent", difficulty: "msDifficulty", word: "msWord",
};

function MistakeBook({ page, query, searchInput, loading, error, canTts, ja, onSearch, onQuery, onMore, onSpeak, t }: {
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

function Stat({ label, value, suffix }: { label: string; value: number; suffix?: string }) {
    return (
        <div className="flex flex-col gap-1">
            <span className="text-2xl font-bold">
                {value}{suffix && <span className="text-sm font-normal ml-0.5">{suffix}</span>}
            </span>
            <span className="text-xs text-neutral-500 dark:text-neutral-400">{label}</span>
        </div>
    );
}

function ErrorNote({ t }: { t: T }) {
    return <p role="alert" className="text-sm text-red-500 text-center">{t("requestError")}</p>;
}
