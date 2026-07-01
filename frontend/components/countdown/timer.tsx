"use client";

import TimerDisplay from './timer-display';
import TimerControls from './timer-controls';
import TimerSettings from './timer-settings';
import useTimer from '@/hooks/useTimer';
import { useAudioBeeper } from '@/hooks/useAudioBeeper';

export default function Timer() {
    const {
        minutes,
        setMinutes,
        timeLeft,
        isRunning,
        isPaused,
        isBeeping,
        startCountdown,
        pauseCountdown,
        resetCountdown,
        stopBeeping,
    } = useTimer();

    const audioRef = useAudioBeeper(isBeeping, stopBeeping, resetCountdown);

    const handleEnterPress = () => {
        if (!isRunning && !isPaused && !isBeeping) startCountdown();
    };

    return (
        <div className="h-[calc(100svh-120px)] overflow-auto flex flex-col items-center justify-center p-4">
            <div className="bg-white dark:bg-neutral-800 shadow-lg rounded-lg p-8 w-full max-w-md">
                <h1 className="text-4xl font-extrabold text-center mb-6 text-primary-600 dark:text-primary-400">倒數計時器</h1>
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
        </div>
    );
}
