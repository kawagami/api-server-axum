"use client";

interface Props {
    max: number;
    selected: number[];
    onToggle: (n: number) => void;
    // 已達上限時，未選的不可再點（單選區傳 false，由 onToggle 自行替換）
    full?: boolean;
}

// 選號按鈕格：1..max 圓鈕，已選填滿 primary，達上限的未選號 disable
export default function NumberGrid({ max, selected, onToggle, full = false }: Props) {
    const sel = new Set(selected);
    return (
        <div className="grid grid-cols-7 gap-1.5 sm:gap-2">
            {Array.from({ length: max }, (_, i) => i + 1).map(n => {
                const isSel = sel.has(n);
                const disabled = !isSel && full;
                return (
                    <button
                        key={n}
                        type="button"
                        onClick={() => onToggle(n)}
                        disabled={disabled}
                        aria-pressed={isSel}
                        className={`aspect-square rounded-full text-sm font-medium flex items-center justify-center transition-colors ${isSel
                            ? 'bg-primary-500 text-white'
                            : disabled
                                ? 'bg-neutral-100 dark:bg-neutral-800 text-neutral-300 dark:text-neutral-600 cursor-not-allowed'
                                : 'bg-neutral-100 dark:bg-neutral-700 hover:bg-primary-100 dark:hover:bg-primary-900'}`}
                    >
                        {n}
                    </button>
                );
            })}
        </div>
    );
}
