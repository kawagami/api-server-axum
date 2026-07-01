import { Link } from "@/i18n/navigation";
import { getTranslations } from "next-intl/server";
import KawaLogo from "@/components/kawa-logo";
import {
    FileText,
    StickyNote,
    Gamepad2,
    Wallet,
    TrendingUp,
    ReceiptText,
    Ticket,
    Wrench,
    Info,
    type LucideIcon,
} from "lucide-react";

// 介紹頁功能卡片單一來源：新增功能只要加一行（icon + 目標路徑 + i18n key）
interface Feature {
    key: string;
    href: string;
    icon: LucideIcon;
}

const FEATURES: Feature[] = [
    { key: "blog", href: "/blogs", icon: FileText },
    { key: "notes", href: "/hackmd-notes", icon: StickyNote },
    { key: "games", href: "/games/chess", icon: Gamepad2 },
    { key: "ledger", href: "/ledger", icon: Wallet },
    { key: "portfolio", href: "/portfolio", icon: TrendingUp },
    { key: "invoices", href: "/invoices", icon: ReceiptText },
    { key: "lotto", href: "/lotto", icon: Ticket },
    { key: "tools", href: "/tools/countdown", icon: Wrench },
    { key: "about", href: "/about", icon: Info },
];

export default async function ProjectIntro() {
    const t = await getTranslations("Home");

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto">
            <div className="max-w-5xl mx-auto px-4 pb-12">
                {/* Hero */}
                <section className="text-center pt-4 pb-10">
                    <div className="flex justify-center mb-4">
                        <KawaLogo width={160} height={64} />
                    </div>
                    <h1 className="text-3xl sm:text-4xl font-bold text-neutral-800 dark:text-neutral-100 mb-3">
                        {t("title")}
                    </h1>
                    <p className="max-w-2xl mx-auto text-neutral-600 dark:text-neutral-300 text-base sm:text-lg mb-3">
                        {t("tagline")}
                    </p>
                    <p className="max-w-2xl mx-auto text-xs sm:text-sm text-neutral-500 dark:text-neutral-400">
                        {t("techStack")}
                    </p>
                </section>

                {/* 功能卡片 */}
                <section className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {FEATURES.map(({ key, href, icon: Icon }) => (
                        <Link
                            key={key}
                            href={href}
                            className="group flex flex-col bg-white dark:bg-neutral-800 shadow-md rounded-xl p-5 hover:shadow-lg transition-shadow duration-300"
                        >
                            <div className="flex items-center gap-3 mb-2">
                                <span className="flex items-center justify-center w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300">
                                    <Icon size={20} />
                                </span>
                                <h2 className="text-lg font-semibold text-neutral-800 dark:text-neutral-100 group-hover:text-primary-600 dark:group-hover:text-primary-300 transition-colors">
                                    {t(`features.${key}.title`)}
                                </h2>
                            </div>
                            <p className="text-sm text-neutral-600 dark:text-neutral-400 leading-relaxed">
                                {t(`features.${key}.desc`)}
                            </p>
                        </Link>
                    ))}
                </section>
            </div>
        </div>
    );
}
