"use client";

import { useState, useEffect, useRef, useCallback } from 'react';
import { Bell, BellOff } from 'lucide-react';
import { startSecondTick } from '@/libs/second-tick';

function msToNextHour(): number {
    const now = new Date();
    const next = new Date(now);
    next.setHours(now.getHours() + 1, 0, 0, 0);
    return next.getTime() - now.getTime();
}

function formatTimeLeft(ms: number): string {
    const totalSec = Math.floor(ms / 1000);
    const m = Math.floor(totalSec / 60).toString().padStart(2, '0');
    const s = (totalSec % 60).toString().padStart(2, '0');
    return `${m}:${s}`;
}

export default function HourlyChime() {
    const [enabled, setEnabled] = useState(false);
    const [currentTime, setCurrentTime] = useState('');
    const [timeLeft, setTimeLeft] = useState('');
    const [lastChime, setLastChime] = useState<string | null>(null);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    function playBell(ctx: AudioContext, startTime: number) {
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.connect(gain);
        gain.connect(ctx.destination);
        osc.type = 'sine';
        osc.frequency.setValueAtTime(1047, startTime);
        osc.frequency.exponentialRampToValueAtTime(880, startTime + 0.1);
        gain.gain.setValueAtTime(1, startTime);
        gain.gain.exponentialRampToValueAtTime(0.001, startTime + 1.2);
        osc.start(startTime);
        osc.stop(startTime + 1.2);
    }

    const chime = useCallback(() => {
        const hour = new Date().getHours();
        const ctx = new AudioContext();
        const gap = 1.4;
        for (let i = 0; i < 3; i++) {
            playBell(ctx, ctx.currentTime + i * gap);
        }
        setTimeout(() => {
            const utterance = new SpeechSynthesisUtterance(`現在 ${hour} 點整`);
            utterance.lang = 'zh-TW';
            utterance.rate = 0.75;
            utterance.volume = 1;
            speechSynthesis.speak(utterance);
        }, 3 * gap * 1000);
        setLastChime(`${hour.toString().padStart(2, '0')}:00`);
    }, []);

    useEffect(() => {
        if (!enabled) {
            speechSynthesis.cancel();
            return;
        }
        function schedule() {
            timerRef.current = setTimeout(() => {
                chime();
                schedule();
            }, msToNextHour());
        }
        schedule();
        return () => {
            if (timerRef.current) clearTimeout(timerRef.current);
        };
    }, [enabled, chime]);

    useEffect(() => {
        return startSecondTick(() => {
            const now = new Date();
            const h = now.getHours().toString().padStart(2, '0');
            const m = now.getMinutes().toString().padStart(2, '0');
            const s = now.getSeconds().toString().padStart(2, '0');
            setCurrentTime(`${h}:${m}:${s}`);
            setTimeLeft(formatTimeLeft(msToNextHour()));
        });
    }, []);

    return (
        <div className="h-[calc(100svh-120px)] overflow-auto flex flex-col items-center justify-center p-4">
            <div className="bg-white dark:bg-neutral-800 shadow-lg rounded-lg p-8 w-full max-w-md flex flex-col items-center gap-6">
                <h1 className="text-4xl font-extrabold text-center text-primary-600 dark:text-primary-400">整點報時</h1>

                <div className="text-6xl font-mono font-bold text-neutral-800 dark:text-neutral-100 tabular-nums">
                    {currentTime || '--:--:--'}
                </div>

                <button
                    onClick={() => setEnabled(v => !v)}
                    className={`flex items-center gap-2 px-6 py-3 rounded-full text-lg font-semibold transition-colors ${
                        enabled
                            ? 'bg-primary-600 hover:bg-primary-700 text-white'
                            : 'bg-neutral-200 dark:bg-neutral-700 hover:bg-neutral-300 dark:hover:bg-neutral-600 text-neutral-800 dark:text-neutral-200'
                    }`}
                >
                    {enabled ? <Bell size={20} /> : <BellOff size={20} />}
                    {enabled ? '報時中' : '已停用'}
                </button>

                <div className="w-full bg-neutral-100 dark:bg-neutral-700 rounded-lg p-4 flex flex-col gap-2 text-sm text-neutral-600 dark:text-neutral-400">
                    <div className="flex justify-between">
                        <span>距下一個整點</span>
                        <span className="font-mono font-semibold text-neutral-800 dark:text-neutral-200">{timeLeft || '--:--'}</span>
                    </div>
                    {lastChime && (
                        <div className="flex justify-between">
                            <span>最近報時</span>
                            <span className="font-mono font-semibold text-neutral-800 dark:text-neutral-200">{lastChime}</span>
                        </div>
                    )}
                </div>

                {!enabled && (
                    <p className="text-xs text-neutral-400 dark:text-neutral-500 text-center">
                        開啟後，每個整點自動播報語音
                    </p>
                )}
            </div>
        </div>
    );
}
