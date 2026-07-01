"use client";

interface Props {
    isRunning: boolean;
    isBeeping: boolean;
    startAlarm: () => void;
    resetAlarm: () => void;
}

export default function AlarmControls({ isRunning, isBeeping, startAlarm, resetAlarm }: Props) {
    return (
        <>
            {!isRunning && !isBeeping && (
                <button onClick={startAlarm} className="w-full px-6 py-3 bg-primary-500 text-white font-semibold rounded-lg shadow-md hover:bg-primary-600 transition focus:outline-none focus:ring-2 focus:ring-primary-400">
                    設定鬧鐘
                </button>
            )}
            {isRunning && (
                <button onClick={resetAlarm} className="w-full px-6 py-3 bg-red-500 text-white font-semibold rounded-lg shadow-md hover:bg-red-600 transition focus:outline-none focus:ring-2 focus:ring-red-400">
                    取消鬧鐘
                </button>
            )}
            {isBeeping && (
                <>
                    <button onClick={resetAlarm} className="w-full px-6 py-3 bg-red-500 text-white font-semibold rounded-lg shadow-md hover:bg-red-600 transition focus:outline-none focus:ring-2 focus:ring-red-400">
                        關閉鬧鐘
                    </button>
                    <p className="text-xl text-red-600 dark:text-red-400 font-medium text-center mt-2">時間到！播放提醒中...</p>
                </>
            )}
        </>
    );
}
