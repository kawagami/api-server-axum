"use client";

import { useRef } from 'react';
import { useTranslations } from 'next-intl';
import { TimerReset } from 'lucide-react';
import useTimer from '@/hooks/useTimer';
import { useAudioBeeper } from '@/hooks/useAudioBeeper';
import TimerCard, { ACTION_BTN } from './timer-card';
import TimeDisplay from './time-display';
import { selectOnHover, NUMBER_INPUT } from './hover-select';

export default function CountdownCard() {
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
    const inputRef = useRef<HTMLInputElement>(null);
    const disabled = isRunning || isPaused || isBeeping;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!disabled) startCountdown();
    };

    return (
        <TimerCard title={t('title')} icon={TimerReset}>
            <form onSubmit={handleSubmit}>
                <label htmlFor="countdown-minutes" className="text-sm font-medium block mb-2 text-neutral-700 dark:text-neutral-300">
                    {t('minutesLabel')}
                </label>
                <input
                    id="countdown-minutes"
                    ref={inputRef}
                    type="number"
                    value={minutes}
                    onChange={(e) => setMinutes(Number(e.target.value))}
                    onFocus={(e) => e.target.select()}
                    onMouseEnter={() => selectOnHover(inputRef.current, disabled)}
                    min="1"
                    max="999"
                    disabled={disabled}
                    className={NUMBER_INPUT}
                />
                <button type="submit" className="hidden" />
            </form>

            <TimeDisplay seconds={timeLeft} />

            {!disabled && (
                <button onClick={startCountdown} className={`${ACTION_BTN} bg-primary-500 hover:bg-primary-600`}>
                    {t('start')}
                </button>
            )}
            {isRunning && (
                <button onClick={pauseCountdown} className={`${ACTION_BTN} bg-yellow-500 hover:bg-yellow-600`}>
                    {t('pause')}
                </button>
            )}
            {isPaused && (
                <button onClick={startCountdown} className={`${ACTION_BTN} bg-green-500 hover:bg-green-600`}>
                    {t('resume')}
                </button>
            )}
            {disabled && (
                <button onClick={resetCountdown} className={`${ACTION_BTN} bg-red-500 hover:bg-red-600`}>
                    {t('reset')}
                </button>
            )}
            {isBeeping && (
                <p className="text-lg text-red-600 dark:text-red-400 font-medium text-center" role="status">
                    {t('ringing')}
                </p>
            )}

            <audio ref={audioRef} src="/beep.mp3" loop />
        </TimerCard>
    );
}
