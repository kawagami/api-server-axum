import type { LucideIcon } from 'lucide-react';

/** 三個計時卡片共用的外框；一頁多顆卡，標題一律 h2（h1 由 PageTitle 出） */
export default function TimerCard({
    title,
    icon: Icon,
    className,
    children,
}: {
    title: string;
    icon: LucideIcon;
    className?: string;
    children: React.ReactNode;
}) {
    return (
        <section className={`bg-white dark:bg-neutral-800 shadow-lg rounded-lg p-6 flex flex-col gap-4 ${className ?? ''}`}>
            <h2 className="flex items-center gap-2 text-lg font-semibold text-neutral-800 dark:text-neutral-100">
                <Icon size={20} className="text-primary-600 dark:text-primary-400" />
                {title}
            </h2>
            {children}
        </section>
    );
}

/** 卡片內主要動作鈕的共用底樣式，顏色由呼叫端接上去 */
export const ACTION_BTN = "w-full px-6 py-3 text-white font-semibold rounded-lg shadow-md transition-colors";
