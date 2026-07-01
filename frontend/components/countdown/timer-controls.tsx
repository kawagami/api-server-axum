"use client";

interface Props {
    isRunning: boolean;
    isPaused: boolean;
    isBeeping: boolean;
    startCountdown: () => void;
    pauseCountdown: () => void;
    resetCountdown: () => void;
}

export default function TimerControls({ isRunning, isPaused, isBeeping, startCountdown, pauseCountdown, resetCountdown }: Props) {
    return (
        <>
            {!isRunning && !isPaused && !isBeeping && (
                <button onClick={startCountdown} className="w-full px-6 py-3 bg-primary-500 text-white font-semibold rounded-lg shadow-md hover:bg-primary-600 transition focus:outline-none focus:ring-2 focus:ring-primary-400">
                    開始倒數
                </button>
            )}
            {isRunning && (
                <button onClick={pauseCountdown} className="w-full px-6 py-3 bg-yellow-500 text-white font-semibold rounded-lg shadow-md hover:bg-yellow-600 transition focus:outline-none focus:ring-2 focus:ring-yellow-400">
                    暫停
                </button>
            )}
            {isPaused && (
                <button onClick={startCountdown} className="w-full px-6 py-3 bg-green-500 text-white font-semibold rounded-lg shadow-md hover:bg-green-600 transition focus:outline-none focus:ring-2 focus:ring-green-400">
                    繼續
                </button>
            )}
            {(isRunning || isPaused || isBeeping) && (
                <button onClick={resetCountdown} className="w-full px-6 py-3 mt-4 bg-red-500 text-white font-semibold rounded-lg shadow-md hover:bg-red-600 transition focus:outline-none focus:ring-2 focus:ring-red-400">
                    重置
                </button>
            )}
            {isBeeping && (
                <p className="text-xl text-red-600 dark:text-red-400 font-medium text-center mt-2">時間到！播放提醒中...</p>
            )}
        </>
    );
}
