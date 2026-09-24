"use client";

import type { VocabRunMode, VocabRunResult } from "@/types";
import { Link } from "@/i18n/navigation";
import { BookOpenCheck, Loader2, LogIn, Sparkles, Trophy } from "lucide-react";
import { Stat, type T } from "./ui";

export function ScoredResultCard({ mode, result, busy, isMember, loginHref, onAgain, onMenu, t }: {
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

export function ReviewResultCard({ result, busy, onAgain, onMenu, t }: {
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
