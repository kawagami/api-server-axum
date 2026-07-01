interface Props {
    main: number[];
    // 第二區/特別號；null = 大樂透登錄注無第二區（開獎結果一律有 special）
    special: number | null;
    size?: 'sm' | 'md';
}

// 號碼球：一般號淺 primary、第二區/特別號實心 primary（以「+」分隔）
export default function Balls({ main, special, size = 'md' }: Props) {
    const dim = size === 'sm' ? 'w-6 h-6 text-[11px]' : 'w-7 h-7 text-xs';
    return (
        <div className="flex flex-wrap items-center gap-1">
            {main.map(n => (
                <span
                    key={n}
                    className={`${dim} rounded-full bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300 font-medium flex items-center justify-center`}
                >
                    {n}
                </span>
            ))}
            {special !== null && (
                <>
                    <span className="text-neutral-300 dark:text-neutral-600 px-0.5">+</span>
                    <span className={`${dim} rounded-full bg-primary-500 text-white font-semibold flex items-center justify-center`}>
                        {special}
                    </span>
                </>
            )}
        </div>
    );
}
