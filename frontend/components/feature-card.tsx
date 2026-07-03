import { Link } from "@/i18n/navigation";
import type { LucideIcon } from "lucide-react";

interface FeatureCardProps {
    href: string;
    icon: LucideIcon;
    title: string;
    desc: string;
}

// 首頁 / 工具 / 遊戲 index 頁共用的功能卡片
export default function FeatureCard({ href, icon: Icon, title, desc }: FeatureCardProps) {
    return (
        <Link
            href={href}
            className="group flex flex-col bg-white dark:bg-neutral-800 shadow-md rounded-xl p-5 hover:shadow-lg transition-shadow duration-300"
        >
            <div className="flex items-center gap-3 mb-2">
                <span className="flex items-center justify-center w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300">
                    <Icon size={20} />
                </span>
                <h2 className="text-lg font-semibold text-neutral-800 dark:text-neutral-100 group-hover:text-primary-600 dark:group-hover:text-primary-300 transition-colors">
                    {title}
                </h2>
            </div>
            <p className="text-sm text-neutral-600 dark:text-neutral-400 leading-relaxed">
                {desc}
            </p>
        </Link>
    );
}
