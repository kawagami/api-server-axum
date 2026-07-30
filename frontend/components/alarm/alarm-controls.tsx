"use client";

import { useTranslations } from 'next-intl';

interface Props {
    isRunning: boolean;
    isBeeping: boolean;
    startAlarm: () => void;
    resetAlarm: () => void;
}

export default function AlarmControls({ isRunning, isBeeping, startAlarm, resetAlarm }: Props) {
    const t = useTranslations('Alarm');

    return (
        <>
            {!isRunning && !isBeeping && (
                <button onClick={startAlarm} className="w-full px-6 py-3 bg-primary-500 text-white font-semibold rounded-lg shadow-md hover:bg-primary-600 transition-colors">
                    {t('start')}
                </button>
            )}
            {isRunning && (
                <button onClick={resetAlarm} className="w-full px-6 py-3 bg-red-500 text-white font-semibold rounded-lg shadow-md hover:bg-red-600 transition-colors">
                    {t('cancel')}
                </button>
            )}
            {isBeeping && (
                <>
                    <button onClick={resetAlarm} className="w-full px-6 py-3 bg-red-500 text-white font-semibold rounded-lg shadow-md hover:bg-red-600 transition-colors">
                        {t('stop')}
                    </button>
                    <p className="text-xl text-red-600 dark:text-red-400 font-medium text-center mt-2" role="status">{t('ringing')}</p>
                </>
            )}
        </>
    );
}
