import { getCurrentMember } from "@/api/members";
import { Link } from "@/i18n/navigation";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { Bell, User, BookOpen, FileText, Wrench, ReceiptText, Ticket } from "lucide-react";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Dashboard");
    return { title: t("title") };
}

const QUICK_LINKS = [
    { href: "/invoices/scan", labelKey: "invoices", descKey: "invoicesDesc", icon: ReceiptText },
    { href: "/lotto/register", labelKey: "lotto", descKey: "lottoDesc", icon: Ticket },
    { href: "/dashboard/notifications", labelKey: "notifications", descKey: "notificationsDesc", icon: Bell },
    { href: "/profile", labelKey: "profile", descKey: "profileDesc", icon: User },
    { href: "/", labelKey: "blogs", descKey: "blogsDesc", icon: BookOpen },
    { href: "/hackmd-notes", labelKey: "notes", descKey: "notesDesc", icon: FileText },
    { href: "/tools/new-password", labelKey: "tools", descKey: "toolsDesc", icon: Wrench },
] as const;

export default async function DashboardPage() {
    const [member, t] = await Promise.all([getCurrentMember(), getTranslations("Dashboard")]);

    return (
        <div className="w-full max-w-3xl px-4 py-8 flex flex-col gap-8">
            <div className="flex items-center gap-4">
                {member.avatar_url ? (
                    // eslint-disable-next-line @next/next/no-img-element
                    <img
                        src={member.avatar_url}
                        alt={member.name}
                        className="w-14 h-14 rounded-full object-cover"
                    />
                ) : (
                    <div className="w-14 h-14 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center text-xl font-bold text-primary-600 dark:text-primary-300">
                        {member.name.charAt(0).toUpperCase()}
                    </div>
                )}
                <div>
                    <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("welcomeBack")}</p>
                    <h1 className="text-2xl font-bold">{member.name}</h1>
                </div>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {QUICK_LINKS.map(({ href, labelKey, descKey, icon: Icon }) => (
                    <Link
                        key={href}
                        href={href}
                        className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow hover:shadow-md transition-shadow flex flex-col gap-2 group"
                    >
                        <Icon size={20} className="text-primary-500 group-hover:scale-110 transition-transform" />
                        <span className="font-semibold text-sm">{t(labelKey)}</span>
                        <span className="text-xs text-neutral-500 dark:text-neutral-400">{t(descKey)}</span>
                    </Link>
                ))}
            </div>
        </div>
    );
}
