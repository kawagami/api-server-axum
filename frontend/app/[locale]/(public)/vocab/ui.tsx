"use client";

import type { VocabMe } from "@/types";
import { Link } from "@/i18n/navigation";
import { Flame, GraduationCap, LogIn, Volume2, VolumeX } from "lucide-react";
import { useTranslations } from "next-intl";
import type { GuestStats } from "./prefs";
import PageTitle from "@/components/page-title";

export type T = ReturnType<typeof useTranslations<"Vocab">>;

export function fmtTime(secs: number) {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
}

/** 發音鈕;裝置沒有語音引擎就整顆不 render(不要留按了沒反應的鈕) */
export function SpeakButton({ text, canTts, onSpeak, t, size = 16 }: {
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

export function ModeButton({ label, desc, best, busy, last, onClick, t }: {
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

export function MuteButton({ muted, onToggle, t }: { muted: boolean; onToggle: () => void; t: T }) {
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
export function VocabHeading({ ja, t }: { ja: boolean; t: T }) {
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

export function GuestBanner({ loginHref, t }: { loginHref: string; t: T }) {
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
export function GuestStatsCard({ stats, t }: { stats: GuestStats; t: T }) {
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

export function LevelCard({ me, t }: { me: VocabMe; t: T }) {
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

export function Stat({ label, value, suffix }: { label: string; value: number; suffix?: string }) {
    return (
        <div className="flex flex-col gap-1">
            <span className="text-2xl font-bold">
                {value}{suffix && <span className="text-sm font-normal ml-0.5">{suffix}</span>}
            </span>
            <span className="text-xs text-neutral-500 dark:text-neutral-400">{label}</span>
        </div>
    );
}

export function ErrorNote({ t }: { t: T }) {
    return <p role="alert" className="text-sm text-red-500 text-center">{t("requestError")}</p>;
}
