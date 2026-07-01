"use client";

import { Link, usePathname } from "@/i18n/navigation";
import { useTranslations } from "next-intl";
import { Ticket, ListChecks, Trophy, Dices, Bell } from "lucide-react";

const LINKS = [
    { href: "/lotto/register", labelKey: "navRegister", icon: Ticket },
    { href: "/lotto", labelKey: "navMyTickets", icon: ListChecks },
    { href: "/lotto/winnings", labelKey: "navWinnings", icon: Trophy },
    { href: "/lotto/draws", labelKey: "navDraws", icon: Dices },
    { href: "/lotto/settings", labelKey: "navSettings", icon: Bell },
] as const;

export default function LottoNav() {
    const t = useTranslations('Lotto');
    const pathname = usePathname();

    return (
        <nav className="flex flex-wrap gap-2 mb-6">
            {LINKS.map(({ href, labelKey, icon: Icon }) => {
                // /lotto 需精確比對，避免被子路由點亮
                const active = href === '/lotto' ? pathname === '/lotto' : pathname.startsWith(href);
                return (
                    <Link
                        key={href}
                        href={href}
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
