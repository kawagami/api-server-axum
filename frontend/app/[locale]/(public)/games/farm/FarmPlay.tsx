"use client";

import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslations } from 'next-intl';
import { RotateCcw, Trophy } from 'lucide-react';
import { unlockSpeech } from '@/libs/speech';
import { sound } from '../_shared/sound';
import { useActionAlert } from '../_shared/useActionAlert';
import type { UseFarmRoom } from './useFarmRoom';
import { useFarmVoice } from './useFarmVoice';
import { composeNarration } from './narration';
import { FarmStatus } from './FarmStatus';
import { FarmActionBoard } from './FarmActionBoard';
import { PlayerPanel } from './FarmBoard';
import { FarmLog } from './FarmLog';

export function FarmPlay({ farm }: { farm: UseFarmRoom }) {
    const t = useTranslations('Farm');
    const { state, room, mySeat, gameOver, notice, log, stateSeq, actions } = farm;
    const voice = useFarmVoice(t('speechLang'));
    const [muted, setMuted] = useState(() => sound.isMuted());

    const seatName = (s: number) => room?.players.find(p => p.seat === s)?.name || t('playerN', { n: s + 1 });
    const myTurn = !!state && state.phase === 'placing' && state.current_player !== null && state.current_player === mySeat;

    // 語音台詞＝最新一批事件 + 現在輪到誰（字幕與「再聽一次」同一段）
    const narration = useMemo(
        () => (state ? composeNarration(t, state, log.filter(e => e.seq === stateSeq), mySeat, seatName) : ''),
        // seatName 每次 render 都是新函式，但只依賴 room.players
        // eslint-disable-next-line react-hooks/exhaustive-deps
        [t, state, log, stateSeq, mySeat, room?.players],
    );

    // 每則新 state 自動念一次（以序號判斷，台詞相同也照念）
    const spokenSeq = useRef(0);
    const { auto } = voice;
    useEffect(() => {
        if (!state || stateSeq === spokenSeq.current) return;
        spokenSeq.current = stateSeq;
        if (narration) auto(narration);
    }, [state, stateSeq, narration, auto]);

    // 動作被 server 拒絕：念出原因（ref 記已念過的，切語音開關不會重念舊錯誤）
    const spokenNotice = useRef<string | null>(null);
    useEffect(() => {
        if (notice === spokenNotice.current) return;
        spokenNotice.current = notice;
        if (notice?.startsWith('err_')) auto(t(notice));
    }, [notice, auto, t]);

    // 結算：念名次與分數（同上，只念一次）
    const rank = gameOver && mySeat !== null
        ? gameOver.scores.filter(s => s > gameOver.scores[mySeat]).length + 1 : null;
    const spokenGameOver = useRef<unknown>(null);
    useEffect(() => {
        if (!gameOver || gameOver === spokenGameOver.current || mySeat === null || rank === null) return;
        spokenGameOver.current = gameOver;
        auto(t('voice_gameOver', { score: gameOver.scores[mySeat], rank }));
    }, [gameOver, mySeat, rank, auto, t]);

    useActionAlert(
        myTurn && state ? `${state.round}-${state.workers_left[mySeat!]}` : null,
        myTurn ? t('yourTurn') : null,
        '🌾',
    );

    if (!state) return null;
    const me = mySeat !== null ? state.players[mySeat] ?? null : null;

    const toggleMute = () => {
        const next = !muted;
        setMuted(next);
        sound.setMuted(next);
    };

    return (
        // 第一次點擊解 autoplay 鎖：Web Audio 音效與 iOS 的 speechSynthesis 都要
        <div className="mx-auto flex w-full max-w-5xl flex-col gap-3 px-2 py-3 sm:px-3"
            onPointerDownCapture={() => { sound.warmup(); unlockSpeech(); }}>
            <FarmStatus state={state} mySeat={mySeat} seatName={seatName} narration={narration} notice={notice}
                voice={voice} muted={muted} onToggleMute={toggleMute} />

            <div className="grid gap-3 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
                <FarmActionBoard key={`${state.round}-${state.current_player}`} state={state} me={me} myTurn={myTurn}
                    seatName={seatName} onAction={actions.sendAction}
                    onCompoundOpen={a => auto(t(`voice_form_${a}`))} />
                <div className="flex flex-col gap-3">
                    {me && mySeat !== null && (
                        <PlayerPanel p={me} seat={mySeat} name={seatName(mySeat)} isMe isCurrent={state.current_player === mySeat}
                            workersLeft={state.workers_left[mySeat] ?? 0} score={state.score_preview[mySeat]} size="lg" />
                    )}
                    <FarmLog log={log} seatName={seatName} />
                </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                {state.players.map((p, seat) => seat !== mySeat && (
                    <PlayerPanel key={seat} p={p} seat={seat} name={seatName(seat)} isMe={false} isCurrent={seat === state.current_player}
                        workersLeft={state.workers_left[seat] ?? 0} score={state.score_preview[seat]} size="sm" />
                ))}
            </div>

            {/* 結算（不可關閉的結局遮罩，不走 Modal） */}
            {gameOver && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-neutral-900/80 p-4 backdrop-blur-xs">
                    <div className="flex w-full max-w-md flex-col gap-3 rounded-lg bg-white p-5 dark:bg-neutral-800">
                        <h2 className="flex items-center justify-center gap-2 text-2xl font-bold text-neutral-800 dark:text-neutral-100">
                            <Trophy className="h-6 w-6 text-amber-500" />{t('gameOverTitle')}
                        </h2>
                        <ul className="flex flex-col gap-1 text-sm">
                            {gameOver.scores
                                .map((score, seat) => ({ score, seat }))
                                .sort((a, b) => b.score - a.score)
                                .map((e, i) => (
                                    <li key={e.seat} className="flex justify-between rounded-sm px-2 py-1 odd:bg-neutral-100 dark:odd:bg-neutral-700">
                                        <span>{i + 1}. {seatName(e.seat)}{e.seat === mySeat && `（${t('youTag')}）`}</span>
                                        <span className="font-mono font-semibold">{e.score}</span>
                                    </li>
                                ))}
                        </ul>
                        <button onClick={actions.backToLobby} className="flex items-center justify-center gap-2 rounded-lg bg-primary-600 px-4 py-2 font-medium text-white transition-colors hover:bg-primary-700">
                            <RotateCcw className="h-4 w-4" />{t('backToLobby')}
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
