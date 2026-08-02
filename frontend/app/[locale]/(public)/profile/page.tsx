import { getCurrentMember } from "@/api/members";
import { getVocabMe } from "@/api/vocab";
import { Link } from "@/i18n/navigation";
import { getLocale, getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { GraduationCap, LayoutDashboard } from "lucide-react";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Profile");
    return { title: t("title") };
}

const PROVIDER_LABELS: Record<string, string> = {
    google: "Google",
    github: "GitHub",
    line: "LINE",
};

export default async function ProfilePage() {
    const [member, vocabMe, t, locale] = await Promise.all([
        getCurrentMember(),
        getVocabMe(),
        getTranslations("Profile"),
        getLocale(),
    ]);
    const levelSpan = vocabMe.next_level_exp - vocabMe.level_exp;
    const levelProgress = levelSpan > 0
        ? Math.min(100, ((vocabMe.exp - vocabMe.level_exp) / levelSpan) * 100)
        : 100;

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle
                title={t("title")}
                actions={
                    <Link
                        href="/dashboard"
                        className="flex items-center gap-1 text-sm text-neutral-500 dark:text-neutral-400 hover:text-primary-500 dark:hover:text-primary-400 transition-colors"
                    >
                        <LayoutDashboard size={16} />
                        {t("dashboard")}
                    </Link>
                }
            />

            <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow-sm flex flex-col gap-5">
                <div className="flex items-center gap-4">
                    {member.avatar_url ? (
                        // eslint-disable-next-line @next/next/no-img-element
                        <img
                            src={member.avatar_url}
                            alt={member.name}
                            className="w-20 h-20 rounded-full object-cover"
                        />
                    ) : (
                        <div className="w-20 h-20 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center text-2xl font-bold text-primary-600 dark:text-primary-300">
                            {member.name.charAt(0).toUpperCase()}
                        </div>
                    )}
                    <div className="flex flex-col gap-1">
                        <span className="text-xl font-semibold">{member.name}</span>
                        {member.email && (
                            <span className="text-sm text-neutral-500 dark:text-neutral-400">{member.email}</span>
                        )}
                    </div>
                </div>

                <div className="flex flex-col gap-2 border-t border-neutral-100 dark:border-neutral-700 pt-4">
                    <div className="flex items-center justify-between">
                        <span className="text-sm text-neutral-500 dark:text-neutral-400">{t("memberLevel")}</span>
                        <Link
                            href="/vocab"
                            className="flex items-center gap-1 text-sm font-semibold text-primary-600 dark:text-primary-400 hover:underline"
                        >
                            <GraduationCap size={16} />
                            Lv.{vocabMe.level}
                        </Link>
                    </div>
                    <div className="h-2 rounded-full bg-neutral-100 dark:bg-neutral-700 overflow-hidden">
                        <div
                            className="h-full rounded-full bg-primary-500"
                            style={{ width: `${levelProgress}%` }}
                        />
                    </div>
                    <span className="text-xs text-neutral-400 dark:text-neutral-500 self-end">
                        {vocabMe.exp} / {vocabMe.next_level_exp} EXP
                    </span>
                </div>

                <div className="flex flex-col gap-2 text-sm text-neutral-500 dark:text-neutral-400 border-t border-neutral-100 dark:border-neutral-700 pt-4">
                    {/* timeZone 必填：server component 跑在容器裡（UTC），少了會與 client 差一天 */}
                    <span>{t("joinedAt")}{new Date(member.created_at).toLocaleDateString(locale, { year: "numeric", month: "long", day: "numeric", timeZone: "Asia/Taipei" })}</span>
                </div>

                {member.providers.length > 0 && (
                    <div className="flex flex-col gap-2 border-t border-neutral-100 dark:border-neutral-700 pt-4">
                        <span className="text-sm text-neutral-500 dark:text-neutral-400">{t("linkedAccounts")}</span>
                        <div className="flex gap-2 flex-wrap">
                            {member.providers.map(p => (
                                <span
                                    key={p}
                                    className="px-3 py-1 text-xs rounded-full bg-neutral-100 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 font-medium"
                                >
                                    {PROVIDER_LABELS[p] ?? p}
                                </span>
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </PageShell>
    );
}
