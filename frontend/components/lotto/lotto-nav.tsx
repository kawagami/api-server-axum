"use client";

import { Ticket, ListChecks, Trophy, Dices, Bell } from "lucide-react";
import SectionNav, { type SectionNavLink } from "@/components/section-nav";

const LINKS: readonly SectionNavLink[] = [
    { href: "/lotto/register", labelKey: "navRegister", icon: Ticket },
    { href: "/lotto", labelKey: "navMyTickets", icon: ListChecks },
    { href: "/lotto/winnings", labelKey: "navWinnings", icon: Trophy },
    { href: "/lotto/draws", labelKey: "navDraws", icon: Dices },
    { href: "/lotto/settings", labelKey: "navSettings", icon: Bell },
];

export default function LottoNav() {
    return <SectionNav namespace="Lotto" rootHref="/lotto" links={LINKS} />;
}
