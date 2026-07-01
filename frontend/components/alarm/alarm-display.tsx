"use client";

export default function AlarmDisplay({ timeLeft, isRunning }: { timeLeft: number; isRunning: boolean }) {
    const hours = Math.floor(timeLeft / 3600);
    const mins = Math.floor((timeLeft % 3600) / 60);
    const secs = timeLeft % 60;

    const formatted = hours > 0
        ? `${String(hours).padStart(2, '0')}:${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
        : `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;

    return (
        <div className="text-6xl font-mono text-center text-primary-800 dark:text-primary-300 bg-neutral-100 dark:bg-neutral-700 p-4 rounded-lg shadow-md mb-6">
            {isRunning ? formatted : '--:--'}
        </div>
    );
}
