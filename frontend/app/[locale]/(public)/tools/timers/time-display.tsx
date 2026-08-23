/** 剩餘秒數顯示：超過一小時才帶時位（鬧鐘會、倒數多半不會） */
export default function TimeDisplay({ seconds, placeholder }: { seconds: number; placeholder?: string }) {
    const pad = (n: number) => String(n).padStart(2, '0');
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    const formatted = hours > 0
        ? `${pad(hours)}:${pad(mins)}:${pad(secs)}`
        : `${pad(mins)}:${pad(secs)}`;

    return (
        <div className="text-5xl sm:text-6xl font-mono tabular-nums text-center text-primary-800 dark:text-primary-300 bg-neutral-100 dark:bg-neutral-700 p-4 rounded-lg shadow-md">
            {placeholder ?? formatted}
        </div>
    );
}
