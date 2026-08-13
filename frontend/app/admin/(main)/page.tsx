import Link from "next/link";
import type { Metadata } from "next";
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
    permission: string;
    feature?: string;
}

function StatCard({ label, value, hint, href, icon: Icon }: Stat) {
    return (
        <Link
            href={href}
            className="flex flex-col gap-2 p-5 bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-700 hover:border-primary-400 dark:hover:border-primary-500 hover:shadow-md transition-[border-color,box-shadow]"
        >
            <div className="flex items-center gap-2 text-neutral-500 dark:text-neutral-400">
                <Icon size={16} />
                <span className="text-sm">{label}</span>
            </div>
            <div className="text-3xl font-bold text-neutral-800 dark:text-neutral-100">
                {value ?? "—"}
            </div>
            <div className="text-xs text-neutral-400 dark:text-neutral-500 min-h-4">
                {hint}
            </div>
        </Link>
    );
}

export default async function AdminDashboardPage() {
    const [permissions, publicSettings, blogs, members, blogComments, contactMessages, wsConns] = await Promise.all([
        getMyPermissions(),
        getPublicSettings(),
        safe(getBlogs({ per_page: 1 })),
        safe(getMembers()),
        safe(getAllBlogComments(1, 1)),
        safe(getContactMessages(1, 1)),
        safe(getWsConnections()),
    ]);
    const enabledFeatures = resolveEnabledFeatures(publicSettings.enabled_features);

    const stats: Stat[] = [
        { label: "文章", value: blogs?.total ?? null, href: "/admin/blogs", icon: FileText, permission: "blog:read", feature: "blog" },
        { label: "文章留言", value: blogComments?.total ?? null, href: "/admin/blog-comments", icon: MessageSquare, permission: "comment:read", feature: "blog" },
        { label: "訪客留言", value: contactMessages?.total ?? null, href: "/admin/messages", icon: Mail, permission: "message:read", feature: "message" },
        { label: "會員", value: members?.total ?? null, href: "/admin/members", icon: Users, permission: "member:read" },
        { label: "線上連線", value: wsConns?.length ?? null, href: "/admin/ws", icon: Radio, permission: "ws:read" },
    ].filter((s) => permissions.includes(s.permission) && isFeatureEnabled(enabledFeatures, s.feature));

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
