"use client";

import AlarmDisplay from './alarm-display';
import AlarmControls from './alarm-controls';
import AlarmSettings from './alarm-settings';
import useAlarm from '@/hooks/useAlarm';
import { useAudioBeeper } from '@/hooks/useAudioBeeper';

export default function Alarm() {
    const {
        hour,
        setHour,
        minute,
        setMinute,
        timeLeft,
        isRunning,
        isBeeping,
        startAlarm,
        resetAlarm,
        stopBeeping,
    } = useAlarm();

    const audioRef = useAudioBeeper(isBeeping, stopBeeping, resetAlarm);

    return (
        <div className="h-[calc(100svh-120px)] overflow-auto flex flex-col items-center justify-center p-4">
            <div className="bg-white dark:bg-neutral-800 shadow-lg rounded-lg p-8 w-full max-w-md">
                <h1 className="text-4xl font-extrabold text-center mb-6 text-primary-600 dark:text-primary-400">鬧鐘</h1>
                <AlarmSettings
                    hour={hour}
                    setHour={setHour}
                    minute={minute}
                    setMinute={setMinute}
                    disabled={isRunning || isBeeping}
                    onEnterPress={startAlarm}
                />
                <AlarmDisplay timeLeft={timeLeft} isRunning={isRunning} />
                <AlarmControls
                    isRunning={isRunning}
                    isBeeping={isBeeping}
                    startAlarm={startAlarm}
                    resetAlarm={resetAlarm}
                />
            </div>
            <audio ref={audioRef} src="/beep.mp3" loop />
        </div>
    );
}
