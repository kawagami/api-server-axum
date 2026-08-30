"use client";

import { ScanLine, ReceiptText, Trophy, Megaphone, Bell } from "lucide-react";
import SectionNav, { type SectionNavLink } from "@/components/section-nav";

const LINKS: readonly SectionNavLink[] = [
    { href: "/invoices/scan", labelKey: "navRegister", icon: ScanLine },
    { href: "/invoices", labelKey: "navMyInvoices", icon: ReceiptText },
    { href: "/invoices/winnings", labelKey: "navWinnings", icon: Trophy },
    { href: "/invoices/draws", labelKey: "navDraws", icon: Megaphone },
    { href: "/invoices/settings", labelKey: "navSettings", icon: Bell },
];

export default function InvoiceNav() {
    return <SectionNav namespace="Invoices" rootHref="/invoices" links={LINKS} />;
}
