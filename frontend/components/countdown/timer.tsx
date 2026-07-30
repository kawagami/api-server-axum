"use client";

import TimerDisplay from './timer-display';
import TimerControls from './timer-controls';
import TimerSettings from './timer-settings';
import useTimer from '@/hooks/useTimer';
import { useAudioBeeper } from '@/hooks/useAudioBeeper';
import { useTranslations } from 'next-intl';
import PageShell from '@/components/page-shell';
import PageTitle from '@/components/page-title';

export default function Timer() {
    const t = useTranslations('Countdown');
    const {
        minutes,
        setMinutes,
        timeLeft,
        targetTime,
        isRunning,
        isPaused,
        isBeeping,
        startCountdown,
        pauseCountdown,
        resetCountdown,
        stopBeeping,
    } = useTimer();

    const audioRef = useAudioBeeper(isBeeping, stopBeeping, resetCountdown, isRunning, targetTime);

    const handleEnterPress = () => {
        if (!isRunning && !isPaused && !isBeeping) startCountdown();
    };

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t('title')} />
            <div className="bg-white dark:bg-neutral-800 shadow-lg rounded-lg p-6 sm:p-8">
                <TimerSettings
                    minutes={minutes}
                    setMinutes={setMinutes}
                    disabled={isRunning || isPaused || isBeeping}
                    onEnterPress={handleEnterPress}
                />
                <TimerDisplay timeLeft={timeLeft} />
                <TimerControls
                    isRunning={isRunning}
                    isPaused={isPaused}
                    isBeeping={isBeeping}
                    startCountdown={startCountdown}
                    pauseCountdown={pauseCountdown}
                    resetCountdown={resetCountdown}
                />
            </div>
            <audio ref={audioRef} src="/beep.mp3" loop />
        </PageShell>
    );
}
