"use client";

import { useRef } from 'react';

interface Props {
    minutes: number;
    setMinutes: (n: number) => void;
    disabled: boolean;
    onEnterPress: () => void;
}

export default function TimerSettings({ minutes, setMinutes, disabled, onEnterPress }: Props) {
    const inputRef = useRef<HTMLInputElement>(null);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!disabled) onEnterPress();
    };

    const handleMouseEnter = () => {
        if (!disabled && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    };

    return (
        <form onSubmit={handleSubmit} className="mb-6">
            <label className="text-lg font-medium block mb-2 text-neutral-700 dark:text-neutral-300">設定分鐘數：</label>
            <div className="relative">
                <input
                    ref={inputRef}
                    type="number"
                    autoFocus
                    onFocus={(e) => e.target.select()}
                    value={minutes}
                    onChange={(e) => setMinutes(Number(e.target.value))}
                    min="1"
                    max="999"
                    disabled={disabled}
                    className="w-full p-3 border border-neutral-300 dark:border-neutral-600 rounded-lg shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-400 focus:border-transparent bg-white dark:bg-neutral-700 text-neutral-900 dark:text-neutral-100"
                    onMouseEnter={handleMouseEnter}
                    onKeyDown={(e) => {
                        if (e.key === 'Enter' && !disabled) {
                            e.preventDefault();
                            onEnterPress();
                        }
                    }}
                />
                {!disabled && <div className="absolute inset-0 pointer-events-none bg-transparent" />}
            </div>
        </form>
    );
}
