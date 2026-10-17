"use client";

import { cn } from "@/libs/cn";

interface Props<T extends string | number> {
    options: { value: T; label: string }[];
    value: T;
    onChange: (value: T) => void;
    disabled?: boolean;
}

/** 單選的一排小膠囊按鈕（區間 / 排序切換） */
export default function Pills<T extends string | number>({ options, value, onChange, disabled }: Props<T>) {
    return (
        <div className="flex gap-1">
            {options.map(o => (
                <button
                    key={o.value}
                    type="button"
                    onClick={() => onChange(o.value)}
                    disabled={disabled}
                    aria-pressed={o.value === value}
                    className={cn(
                        "px-3 py-1 rounded-full text-xs border transition-colors disabled:opacity-50",
                        o.value === value
                            ? "border-primary-500 bg-primary-500 text-white"
                            : "border-neutral-200 dark:border-neutral-600 hover:border-primary-400",
                    )}
                >
                    {o.label}
                </button>
            ))}
        </div>
    );
}
