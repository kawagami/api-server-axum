"use client";

import { Link, usePathname } from "@/i18n/navigation";
import { useTranslations } from "next-intl";
import type { LucideIcon } from "lucide-react";

export interface SectionNavLink {
    /** locale prefix 不用寫，@/i18n/navigation 的 Link 會補 */
    href: string;
    /** namespace 底下的 key */
    labelKey: string;
    icon: LucideIcon;
}

/**
 * 會員功能區（發票 / 樂透）的區內導覽膠囊列。
 *
 * active 判定內建：`rootHref` 必須精確比對，否則 `/invoices` 會被 `/invoices/scan`
 * 一起點亮；其餘用 `startsWith`，之後若長出子路由也算在該項下。
 *
 * **不自帶外距** —— 間距由頁面 `PageShell className="flex flex-col gap-6"` 給，
 * 元件自己加 `mb-*` 會與其他頁的節奏對不上。
 *
 * 呼叫端是 `components/{invoices,lotto}/*-nav.tsx` 那兩支設定檔：它們必須是
 * client component —— `icon` 是元件參考、不可序列化，從 server component
 * 傳進 client 會炸。
 */
export default function SectionNav({
    namespace,
    rootHref,
    links,
}: {
    namespace: string;
    rootHref: string;
    links: readonly SectionNavLink[];
}) {
    const t = useTranslations(namespace);
    const pathname = usePathname();

    return (
        <nav className="flex flex-wrap gap-2">
            {links.map(({ href, labelKey, icon: Icon }) => {
                const active = href === rootHref ? pathname === rootHref : pathname.startsWith(href);
                return (
                    <Link
                        key={href}
                        href={href}
                        aria-current={active ? "page" : undefined}
                        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-sm transition-colors ${active
                            ? 'bg-primary-500 text-white'
                            : 'border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}`}
                    >
                        <Icon size={15} />
                        {t(labelKey)}
                    </Link>
                );
            })}
        </nav>
    );
}
