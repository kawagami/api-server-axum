"use client";

import { useRef } from 'react';
import { useTranslations } from 'next-intl';
import { AlarmClock } from 'lucide-react';
import useAlarm from '@/hooks/useAlarm';
import { useAudioBeeper } from '@/hooks/useAudioBeeper';
import TimerCard, { ACTION_BTN } from './timer-card';
import TimeDisplay from './time-display';
import { selectOnHover, NUMBER_INPUT } from './hover-select';

export default function AlarmCard() {
    const t = useTranslations('Alarm');
    const {
        hour,
        setHour,
        minute,
        setMinute,
        timeLeft,
        targetTime,
        isRunning,
        isBeeping,
        startAlarm,
        resetAlarm,
        stopBeeping,
    } = useAlarm();

    const audioRef = useAudioBeeper(isBeeping, stopBeeping, resetAlarm, isRunning, targetTime);
    const hourRef = useRef<HTMLInputElement>(null);
    const minuteRef = useRef<HTMLInputElement>(null);
    const disabled = isRunning || isBeeping;

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!disabled) startAlarm();
    };

    return (
        <TimerCard title={t('title')} icon={AlarmClock}>
            <form onSubmit={handleSubmit}>
                <label htmlFor="alarm-hour" className="text-sm font-medium block mb-2 text-neutral-700 dark:text-neutral-300">
                    {t('timeLabel')}
                </label>
                <div className="flex items-center gap-2">
                    <input
                        id="alarm-hour"
                        ref={hourRef}
                        type="number"
                        value={hour}
                        onChange={(e) => setHour(Math.min(23, Math.max(0, Number(e.target.value))))}
                        onFocus={(e) => e.target.select()}
                        onMouseEnter={() => selectOnHover(hourRef.current, disabled)}
                        min="0"
                        max="23"
                        disabled={disabled}
                        aria-label={t('timeLabel')}
                        className={`${NUMBER_INPUT} text-center text-xl`}
                    />
                    <span className="text-2xl font-bold text-neutral-700 dark:text-neutral-300">:</span>
                    <input
                        ref={minuteRef}
                        type="number"
                        value={minute}
                        onChange={(e) => setMinute(Math.min(59, Math.max(0, Number(e.target.value))))}
                        onFocus={(e) => e.target.select()}
                        onMouseEnter={() => selectOnHover(minuteRef.current, disabled)}
                        min="0"
                        max="59"
                        disabled={disabled}
                        aria-label={t('timeLabel')}
                        className={`${NUMBER_INPUT} text-center text-xl`}
                    />
                </div>
                <p className="mt-2 text-sm text-neutral-500 dark:text-neutral-400 text-center">
                    {t('setAt', { time: `${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}` })}
                </p>
                <button type="submit" className="hidden" />
            </form>

            <TimeDisplay seconds={timeLeft} placeholder={isRunning ? undefined : '--:--'} />

            {!disabled && (
                <button onClick={startAlarm} className={`${ACTION_BTN} bg-primary-500 hover:bg-primary-600`}>
                    {t('start')}
                </button>
            )}
            {isRunning && (
                <button onClick={resetAlarm} className={`${ACTION_BTN} bg-red-500 hover:bg-red-600`}>
                    {t('cancel')}
                </button>
            )}
            {isBeeping && (
                <>
                    <button onClick={resetAlarm} className={`${ACTION_BTN} bg-red-500 hover:bg-red-600`}>
                        {t('stop')}
                    </button>
                    <p className="text-lg text-red-600 dark:text-red-400 font-medium text-center" role="status">
                        {t('ringing')}
                    </p>
                </>
            )}

            <audio ref={audioRef} src="/beep.mp3" loop />
        </TimerCard>
    );
}
