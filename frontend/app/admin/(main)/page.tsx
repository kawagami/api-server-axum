import Link from "next/link";
import { FileText, Users, Image as ImageIcon, Radio, Gamepad2, type LucideIcon } from "lucide-react";
import { getBlogs } from "@/api/blogs";
import { getMembers } from "@/api/members";
import { getImages } from "@/api/images";
import { getWsConnections } from "@/api/ws";
import { getGamesOverview } from "@/api/games";
import { adminNavGroups } from "@/components/admin/nav";

// 單一端點掛掉不應整頁白屏：取值失敗回 null（顯示 —）
async function safe<T>(p: Promise<T>): Promise<T | null> {
    try {
        return await p;
    } catch {
        return null;
    }
}

interface Stat {
    label: string;
    value: number | null;
    hint?: string;
    href: string;
    icon: LucideIcon;
}

function StatCard({ label, value, hint, href, icon: Icon }: Stat) {
    return (
        <Link
            href={href}
            className="flex flex-col gap-2 p-5 bg-white dark:bg-neutral-900 rounded-lg shadow border border-neutral-200 dark:border-neutral-700 hover:border-primary-400 dark:hover:border-primary-500 hover:shadow-md transition-colors"
        >
            <div className="flex items-center gap-2 text-neutral-500 dark:text-neutral-400">
                <Icon size={16} />
                <span className="text-sm">{label}</span>
            </div>
            <div className="text-3xl font-bold text-neutral-800 dark:text-neutral-100">
                {value ?? "—"}
            </div>
            <div className="text-xs text-neutral-400 dark:text-neutral-500 min-h-[1rem]">
                {hint}
            </div>
        </Link>
    );
}

export default async function AdminDashboardPage() {
    const [blogs, members, images, wsConns, games] = await Promise.all([
        safe(getBlogs({ per_page: 1 })),
        safe(getMembers()),
        safe(getImages()),
        safe(getWsConnections()),
        safe(getGamesOverview()),
    ]);

    const unusedImages = images?.filter((i) => i.status === "unused").length ?? 0;
    const playersInGame = games?.reduce((sum, g) => sum + g.players_in_game, 0) ?? null;

    const stats: Stat[] = [
        { label: "文章", value: blogs?.total ?? null, href: "/admin/blogs", icon: FileText },
        { label: "會員", value: members?.length ?? null, href: "/admin/members", icon: Users },
        {
            label: "圖片",
            value: images?.length ?? null,
            hint: unusedImages > 0 ? `${unusedImages} 張待清除` : undefined,
            href: "/admin/images",
            icon: ImageIcon,
        },
        { label: "線上連線", value: wsConns?.length ?? null, href: "/admin/ws", icon: Radio },
        { label: "對局中人數", value: playersInGame, href: "/admin/games", icon: Gamepad2 },
    ];

    return (
        <div className="max-w-5xl mx-auto flex flex-col gap-8">
            <h1 className="text-2xl font-bold text-neutral-800 dark:text-neutral-100">
                Admin Dashboard
            </h1>

            {/* 即時統計快照 */}
            <section className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4">
                {stats.map((s) => (
                    <StatCard key={s.label} {...s} />
                ))}
            </section>

            {/* 依分組的快速入口 */}
            <section className="flex flex-col gap-6">
                {adminNavGroups.map((group) => {
                    const Icon = group.icon;
                    return (
                        <div key={group.label} className="flex flex-col gap-3">
                            <h2 className="flex items-center gap-2 text-sm font-semibold text-neutral-500 dark:text-neutral-400">
                                <Icon size={16} />
                                {group.label}
                            </h2>
                            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3">
                                {group.items.map((item) => (
                                    <Link
                                        key={item.href}
                                        href={item.href}
                                        className="px-4 py-3 text-sm font-medium text-neutral-700 dark:text-neutral-200 bg-white dark:bg-neutral-900 rounded-lg border border-neutral-200 dark:border-neutral-700 hover:border-primary-400 dark:hover:border-primary-500 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                                    >
                                        {item.label}
                                    </Link>
                                ))}
                            </div>
                        </div>
                    );
                })}
            </section>

            <p className="text-xs text-neutral-400 dark:text-neutral-500">
                統計為載入當下快照；對局中人數為記憶體即時值，伺服器重啟後歸零。
            </p>
        </div>
    );
}
