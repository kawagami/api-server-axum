"use client";

export default function TimerDisplay({ timeLeft }: { timeLeft: number }) {
    const formatTime = (seconds: number) => {
        const mins = Math.floor(seconds / 60);
        const secs = seconds % 60;
        return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
    };

    return (
        <div className="text-6xl font-mono text-center text-primary-800 dark:text-primary-300 bg-neutral-100 dark:bg-neutral-700 p-4 rounded-lg shadow-md mb-6">
            {formatTime(timeLeft)}
        </div>
    );
}
