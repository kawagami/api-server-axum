"use client";

import AlarmDisplay from './alarm-display';
import AlarmControls from './alarm-controls';
import AlarmSettings from './alarm-settings';
import useAlarm from '@/hooks/useAlarm';
import { useAudioBeeper } from '@/hooks/useAudioBeeper';
import { useTranslations } from 'next-intl';
import PageShell from '@/components/page-shell';
import PageTitle from '@/components/page-title';

export default function Alarm() {
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

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t('title')} />
            <div className="bg-white dark:bg-neutral-800 shadow-lg rounded-lg p-6 sm:p-8">
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
        </PageShell>
    );
}
