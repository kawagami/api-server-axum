"use client";

import { useTranslations } from 'next-intl';

interface Props {
    isRunning: boolean;
    isPaused: boolean;
    isBeeping: boolean;
    startCountdown: () => void;
    pauseCountdown: () => void;
    resetCountdown: () => void;
}

export default function TimerControls({ isRunning, isPaused, isBeeping, startCountdown, pauseCountdown, resetCountdown }: Props) {
    const t = useTranslations('Countdown');

    return (
        <>
            {!isRunning && !isPaused && !isBeeping && (
                <button onClick={startCountdown} className="w-full px-6 py-3 bg-primary-500 text-white font-semibold rounded-lg shadow-md hover:bg-primary-600 transition-colors">
                    {t('start')}
                </button>
            )}
            {isRunning && (
                <button onClick={pauseCountdown} className="w-full px-6 py-3 bg-yellow-500 text-white font-semibold rounded-lg shadow-md hover:bg-yellow-600 transition-colors">
                    {t('pause')}
                </button>
            )}
            {isPaused && (
                <button onClick={startCountdown} className="w-full px-6 py-3 bg-green-500 text-white font-semibold rounded-lg shadow-md hover:bg-green-600 transition-colors">
                    {t('resume')}
                </button>
            )}
            {(isRunning || isPaused || isBeeping) && (
                <button onClick={resetCountdown} className="w-full px-6 py-3 mt-4 bg-red-500 text-white font-semibold rounded-lg shadow-md hover:bg-red-600 transition-colors">
                    {t('reset')}
                </button>
            )}
            {isBeeping && (
                <p className="text-xl text-red-600 dark:text-red-400 font-medium text-center mt-2" role="status">{t('ringing')}</p>
            )}
        </>
    );
}
