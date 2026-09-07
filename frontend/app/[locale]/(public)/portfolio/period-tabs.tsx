"use client";

/**
 * 期間切換。泛型是因為兩個呼叫端的選項不同：總覽是今日/近一週/近一月，
 * 歷史表格是近一週/近一月/全部（「今日」在逐日表格裡只有一列，沒有意義）。
 */
export default function PeriodTabs<T extends string>({ options, value, onChange, label }: {
    options: readonly T[];
    value: T;
    onChange: (v: T) => void;
    label: (v: T) => string;
}) {
    return (
        <div className="inline-flex rounded-lg border dark:border-neutral-700 bg-white dark:bg-neutral-800 p-0.5">
            {options.map(o => (
                <button
                    key={o}
                    onClick={() => onChange(o)}
                    aria-pressed={o === value}
                    className={`px-3 py-1 text-xs rounded-md transition-colors ${
                        o === value
                            ? 'bg-primary-500 text-white'
                            : 'text-neutral-500 dark:text-neutral-400 hover:text-primary-500 dark:hover:text-primary-400'
                    }`}
                >
                    {label(o)}
                </button>
            ))}
        </div>
    );
}
