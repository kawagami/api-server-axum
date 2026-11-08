import { Suspense } from "react";
import Link from "next/link";
import type { Metadata } from "next";
import { unstable_rethrow } from "next/navigation";
import { FileText, Users, MessageSquare, Radio, Mail, type LucideIcon } from "lucide-react";
import PageHeader from "@/components/admin/page-header";
import { getBlogs } from "@/api/blogs";
import { getMembers } from "@/api/members";
import { getAllBlogComments } from "@/api/blog-comments";
import { getContactMessages } from "@/api/contact";
import { getWsConnections } from "@/api/ws";
import { adminNavGroups, filterNavByPermissions } from "@/components/admin/nav";
import { getMyPermissions } from "@/libs/admin-permissions";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";

export const metadata: Metadata = {
    title: "儀表板",
    description: "後台總覽",
};

const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse";

interface Stat {
    label: string;
    href: string;
    icon: LucideIcon;
    permission: string;
    feature?: string;
    // 延遲取值：只有通過權限／功能開關過濾的卡片才會真的呼叫，沒權限的端點不打（避免白打 403）
    load: () => Promise<number>;
}

const STATS: Stat[] = [
    { label: "文章", href: "/admin/blogs", icon: FileText, permission: "blog:read", feature: "blog",
        load: async () => (await getBlogs({ per_page: 1 })).total },
    { label: "文章留言", href: "/admin/blog-comments", icon: MessageSquare, permission: "comment:read", feature: "blog",
        load: async () => (await getAllBlogComments(1, 1)).total },
    { label: "訪客留言", href: "/admin/messages", icon: Mail, permission: "message:read", feature: "message",
        load: async () => (await getContactMessages(1, 1)).total },
    { label: "會員", href: "/admin/members", icon: Users, permission: "member:read",
        load: async () => (await getMembers()).total },
    { label: "線上連線", href: "/admin/ws", icon: Radio, permission: "ws:read",
        load: async () => (await getWsConnections()).length },
];

// 數值列與提示列的高度（text-3xl 行高 = h-9、提示列 h-4）：骨架、成功、失敗三態共用，串流替換時卡片不跳
const VALUE_ROW = "h-9 text-3xl font-bold text-neutral-800 dark:text-neutral-100";
const HINT_ROW = "h-4 text-xs";

/**
 * 單卡取值。各卡包自己的 Suspense：慢的端點只讓那張卡停在骨架，不拖住整頁。
 * 失敗與「0」要分得出來：失敗顯示 — 並在提示列標「載入失敗」。
 */
async function StatValue({ load }: { load: Stat["load"] }) {
    let value: number | null = null;
    try {
        value = await load();
    } catch (e) {
        // adminRequest 401 丟的是 Next redirect，吞掉就不會導回登入頁
        unstable_rethrow(e);
    }
    return (
        <>
            <div className={VALUE_ROW}>{value ?? "—"}</div>
            <div className={HINT_ROW}>
                {value === null && <span className="text-red-500">載入失敗</span>}
            </div>
        </>
    );
}

function StatValueSkeleton() {
    return (
        <>
            <div className="h-9 flex items-center">
                <div className={`h-7 w-12 ${PULSE}`} />
            </div>
            <div className={HINT_ROW} />
        </>
    );
}

function StatCard({ label, href, icon: Icon, load }: Stat) {
    return (
        <Link
            href={href}
            className="flex flex-col gap-2 p-5 bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-700 hover:border-primary-400 dark:hover:border-primary-500 hover:shadow-md transition-[border-color,box-shadow]"
        >
            <div className="flex items-center gap-2 text-neutral-500 dark:text-neutral-400">
                <Icon size={16} />
                <span className="text-sm">{label}</span>
            </div>
            <Suspense fallback={<StatValueSkeleton />}>
                <StatValue load={load} />
            </Suspense>
        </Link>
    );
}

export default async function AdminDashboardPage() {
    // 兩者都有快取（getCurrentAdmin 走 React cache、layout 已打過；settings 60s cache），頁框立即出來
    const [permissions, publicSettings] = await Promise.all([
        getMyPermissions(),
        getPublicSettings(),
    ]);
    const enabledFeatures = resolveEnabledFeatures(publicSettings.enabled_features);

    const stats = STATS.filter(
        (s) => permissions.includes(s.permission) && isFeatureEnabled(enabledFeatures, s.feature),
    );

    const navGroups = filterNavByPermissions(adminNavGroups, permissions, enabledFeatures);

    return (
        <div className="flex flex-col gap-8">
            <PageHeader title="儀表板" />

            {/* 即時統計快照 */}
            <section className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4">
                {stats.map((s) => (
                    <StatCard key={s.label} {...s} />
                ))}
            </section>

            {/* 依分組的快速入口 */}
            <section className="flex flex-col gap-6">
                {navGroups.map((group) => {
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
                統計為載入當下快照；線上連線為記憶體即時值，伺服器重啟後歸零。
            </p>
        </div>
    );
}
