"use client";

import { useState, useEffect, useRef, useCallback } from 'react';
import { useTranslations } from 'next-intl';
import { PersonStanding } from 'lucide-react';
import { startSecondTick } from '@/libs/second-tick';
import { speakText, stopSpeaking, unlockSpeech } from '@/libs/speech';
import TimerCard, { ACTION_BTN } from './timer-card';
import TimeDisplay from './time-display';
import { selectOnHover, NUMBER_INPUT } from './hover-select';

type Phase = 'idle' | 'sitting' | 'moving';

/** 兩個分鐘數存 localStorage：每天都要開的提醒，重整就要重設很煩。版號在 key 裡，改形狀直接換號 */
const STORAGE_KEY = 'sedentary_settings_v1';
const SIT_RANGE: [number, number] = [1, 180];
const MOVE_RANGE: [number, number] = [1, 60];
const DEFAULT_SIT = 20;
const DEFAULT_MOVE = 5;

/** 欄位存字串（清空欄位那一刻不會被搶成 1），開始時才夾回範圍 */
function clampMinutes(raw: string, [min, max]: [number, number], fallback: number): number {
    const parsed = parseInt(raw, 10);
    if (Number.isNaN(parsed)) return fallback;
    return Math.min(max, Math.max(min, parsed));
}

function playTone(ctx: AudioContext, freq: number, startTime: number) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.type = 'sine';
    osc.frequency.setValueAtTime(freq, startTime);
    gain.gain.setValueAtTime(0.8, startTime);
    gain.gain.exponentialRampToValueAtTime(0.001, startTime + 0.6);
    osc.start(startTime);
    osc.stop(startTime + 0.6);
}

/**
 * 久坐提醒：坐 N 分鐘 → 提醒起來動 → 活動 M 分鐘 → 提醒坐回去，循環到按停止。
 * 不用 useCountdownCore：那支到點會轉成「響鈴中、等人按停」，這裡要的是自動接下一段、不必人介入。
 * 提醒用短音 + 語音 + 系統通知（不用 beep.mp3 循環播放：每 20 分鐘響兩分鐘太吵，而且要自動進下一段）。
 */
export default function SedentaryCard() {
    const t = useTranslations('Sedentary');
    const [sitMin, setSitMin] = useState(String(DEFAULT_SIT));
    const [moveMin, setMoveMin] = useState(String(DEFAULT_MOVE));
    const [phase, setPhase] = useState<Phase>('idle');
    const [endAt, setEndAt] = useState<number | null>(null);
    const [timeLeft, setTimeLeft] = useState(DEFAULT_SIT * 60);
    const [rounds, setRounds] = useState(0);
    /** 開始那一刻定下的秒數；跑的期間欄位鎖住，用 ref 免得換段時讀到 stale closure */
    const durations = useRef({ sit: DEFAULT_SIT * 60, move: DEFAULT_MOVE * 60 });
    /** 在「開始」的點擊裡建立：自動播放政策下，事後在 timer callback 裡才建的 context 可能被擋 */
    const audioCtx = useRef<AudioContext | null>(null);
    const loaded = useRef(false);
    const sitRef = useRef<HTMLInputElement>(null);
    const moveRef = useRef<HTMLInputElement>(null);
    const running = phase !== 'idle';

    useEffect(() => {
        try {
            const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null') as Record<string, unknown> | null;
            // localStorage 是 client-only，mount 後才讀得到（在 render 期讀會 hydration 不一致）
            if (typeof saved?.sit === 'string') {
                // eslint-disable-next-line react-hooks/set-state-in-effect
                setSitMin(saved.sit);
                setTimeLeft(clampMinutes(saved.sit, SIT_RANGE, DEFAULT_SIT) * 60);
            }
            if (typeof saved?.move === 'string') setMoveMin(saved.move);
        } catch {
            // 壞掉的存檔就用預設值
        }
        loaded.current = true;
    }, []);

    useEffect(() => {
        if (!loaded.current) return;
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify({ sit: sitMin, move: moveMin }));
        } catch {
            // 私密瀏覽或配額滿：存不進去不影響功能
        }
    }, [sitMin, moveMin]);

    const announce = useCallback((next: Exclude<Phase, 'idle'>) => {
        const text = next === 'moving' ? t('speechMove') : t('speechSit');
        const ctx = audioCtx.current;
        if (ctx) {
            if (ctx.state === 'suspended') ctx.resume().catch(() => {});
            // 起來動：上行三音；坐回去：下行三音，不看畫面也聽得出是哪一段
            const freqs = next === 'moving' ? [660, 880, 1047] : [1047, 880, 660];
            freqs.forEach((f, i) => playTone(ctx, f, ctx.currentTime + i * 0.35));
        }
        // 排隊不打斷：同頁的整點報時可能剛好同時開口
        setTimeout(() => speakText(text, t('speechLang'), 0.9, { queue: true }), 1200);
        // 人在別的視窗才發系統通知；document.hidden 抓不到 Alt-Tab 切到別的 App（分頁仍是 visible）
        if ('Notification' in window && Notification.permission === 'granted' && !document.hasFocus()) {
            try {
                new Notification(t('title'), { body: text, tag: 'sedentary' });
            } catch {
                // Android Chrome 不支援建構式（要走 Service Worker），只能靠聲音
            }
        }
    }, [t]);

    // 換段用單發的 setTimeout 對準 endAt，不跟顯示用的每秒 tick 綁在一起：
    // 分頁在背景超過 5 分鐘後 Chrome 會把「連鎖」timer 節流到每分鐘一次，tick 會遲到最多一分鐘，單發的不受影響。
    useEffect(() => {
        if (phase === 'idle' || endAt === null) return;
        const timer = setTimeout(() => {
            const next = phase === 'sitting' ? 'moving' : 'sitting';
            announce(next);
            if (phase === 'moving') setRounds(r => r + 1);
            setPhase(next);
            setEndAt(Date.now() + durations.current[next === 'sitting' ? 'sit' : 'move'] * 1000);
        }, Math.max(0, endAt - Date.now()));
        return () => clearTimeout(timer);
    }, [phase, endAt, announce]);

    useEffect(() => {
        if (endAt === null) return;
        return startSecondTick(() => {
            setTimeLeft(Math.max(0, Math.round((endAt - Date.now()) / 1000)));
        }, endAt);
    }, [endAt]);

    const start = () => {
        const sit = clampMinutes(sitMin, SIT_RANGE, DEFAULT_SIT);
        const move = clampMinutes(moveMin, MOVE_RANGE, DEFAULT_MOVE);
        setSitMin(String(sit));
        setMoveMin(String(move));
        durations.current = { sit: sit * 60, move: move * 60 };
        audioCtx.current ??= new AudioContext();
        unlockSpeech(); // iOS：之後在 timer 裡自動念的前提是先在使用者手勢內念過一次
        if ('Notification' in window && Notification.permission === 'default') {
            Notification.requestPermission().catch(() => {});
        }
        setRounds(0);
        setPhase('sitting');
        setEndAt(Date.now() + sit * 60 * 1000);
    };

    /** 跳到下一段（提早起來動、或活動完提早坐回去），不出聲 */
    const skip = () => {
        const next = phase === 'sitting' ? 'moving' : 'sitting';
        if (phase === 'moving') setRounds(r => r + 1);
        setPhase(next);
        setEndAt(Date.now() + durations.current[next === 'sitting' ? 'sit' : 'move'] * 1000);
    };

    const stop = () => {
        stopSpeaking();
        setPhase('idle');
        setEndAt(null);
        setTimeLeft(clampMinutes(sitMin, SIT_RANGE, DEFAULT_SIT) * 60);
    };

    const updateSit = (value: string) => {
        setSitMin(value);
        setTimeLeft(clampMinutes(value, SIT_RANGE, DEFAULT_SIT) * 60);
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (!running) start();
    };

    return (
        <TimerCard title={t('title')} icon={PersonStanding}>
            <div className="grid gap-6 md:grid-cols-2">
                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <label htmlFor="sedentary-sit" className="text-sm font-medium block mb-2 text-neutral-700 dark:text-neutral-300">
                                {t('sitLabel')}
                            </label>
                            <input
                                id="sedentary-sit"
                                ref={sitRef}
                                type="number"
                                value={sitMin}
                                onChange={(e) => updateSit(e.target.value)}
                                onFocus={(e) => e.target.select()}
                                onMouseEnter={() => selectOnHover(sitRef.current, running)}
                                min={SIT_RANGE[0]}
                                max={SIT_RANGE[1]}
                                disabled={running}
                                className={NUMBER_INPUT}
                            />
                        </div>
                        <div>
                            <label htmlFor="sedentary-move" className="text-sm font-medium block mb-2 text-neutral-700 dark:text-neutral-300">
                                {t('moveLabel')}
                            </label>
                            <input
                                id="sedentary-move"
                                ref={moveRef}
                                type="number"
                                value={moveMin}
                                onChange={(e) => setMoveMin(e.target.value)}
                                onFocus={(e) => e.target.select()}
                                onMouseEnter={() => selectOnHover(moveRef.current, running)}
                                min={MOVE_RANGE[0]}
                                max={MOVE_RANGE[1]}
                                disabled={running}
                                className={NUMBER_INPUT}
                            />
                        </div>
                    </div>
                    <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('hint')}</p>
                    <button type="submit" className="hidden" />
                </form>

                <div className="flex flex-col gap-4">
                    <div className="flex items-center justify-between text-sm" role="status">
                        <span
                            className={`px-3 py-1 rounded-full font-semibold ${
                                phase === 'moving'
                                    ? 'bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300'
                                    : phase === 'sitting'
                                        ? 'bg-primary-100 text-primary-800 dark:bg-primary-900/40 dark:text-primary-300'
                                        : 'bg-neutral-100 text-neutral-600 dark:bg-neutral-700 dark:text-neutral-400'
                            }`}
                        >
                            {t(`phase.${phase}`)}
                        </span>
                        {running && (
                            <span className="text-neutral-600 dark:text-neutral-400">{t('rounds', { count: rounds })}</span>
                        )}
                    </div>

                    <TimeDisplay seconds={timeLeft} />

                    {!running ? (
                        <button onClick={start} className={`${ACTION_BTN} bg-primary-500 hover:bg-primary-600`}>
                            {t('start')}
                        </button>
                    ) : (
                        <div className="grid grid-cols-2 gap-4">
                            <button onClick={skip} className={`${ACTION_BTN} bg-yellow-500 hover:bg-yellow-600`}>
                                {phase === 'sitting' ? t('skipToMove') : t('skipToSit')}
                            </button>
                            <button onClick={stop} className={`${ACTION_BTN} bg-red-500 hover:bg-red-600`}>
                                {t('stop')}
                            </button>
                        </div>
                    )}
                </div>
            </div>
        </TimerCard>
    );
}
