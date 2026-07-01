"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { Flag, RotateCcw, Volume2, VolumeX } from 'lucide-react';
import { Clock } from './Clock';
import { Lobby } from './Lobby';
import { sound } from './sound';
import type { UseGameRoom } from './useGameRoom';

// 對局外框（時鐘 + 盤位 + 控制 + 結束遮罩），大廳系列委派 Lobby。
// 文案走 GameLobby namespace；各遊戲提供 title / 側標籤 / dot 配色 / reason 文字 / 盤面節點。
export function GameFrame({
    room, title, rules, sides, sideLabel, sideDotClass, reasonLabel, board, extraStatus, extraControls,
}: {
    room: UseGameRoom;
    title: string;
    rules: string[];
    sides: readonly [string, string];
    sideLabel: (side: string) => string;
    sideDotClass: (side: string) => string;
    reasonLabel: (reason: string) => string;
    board: React.ReactNode;
    extraStatus?: React.ReactNode;   // 例如象棋將軍提示
    extraControls?: React.ReactNode; // 例如圍棋 pass 鈕（置於控制列）
}) {
    const t = useTranslations('GameLobby');
    const [muted, setMuted] = useState(false);
    const { phase, myColor, turn, clock, result, shake, actions } = room;

    if (phase === 'connecting' || phase === 'lobby' || phase === 'queued' || phase === 'hosting') {
        return (
            <Lobby
                phase={phase}
                title={title}
                rules={rules}
                tables={room.tables}
                queuePos={room.queuePos}
                hostedTableId={room.hostedTableId}
                notice={room.notice}
                onQuickMatch={actions.quickMatch}
                onCreateTable={actions.createTable}
                onJoinTable={actions.joinTable}
                onLeaveQueue={actions.leaveQueue}
                onCancelHost={actions.cancelHost}
            />
        );
    }

    const oppColor = sides[0] === myColor ? sides[1] : sides[0];
    const myTurn = phase === 'playing' && turn === myColor;

    const toggleMute = () => {
        const next = !muted;
        setMuted(next);
        sound.setMuted(next);
    };

    const resultTitle = !result ? '' : result.winner === null ? t('draw') : result.winner === myColor ? t('win') : t('lose');

    return (
        <div className="mx-auto flex h-[calc(100svh-120px)] w-full max-w-xl flex-col items-center gap-3 overflow-hidden py-2 lg:max-w-2xl xl:max-w-3xl">
            {/* 對手時鐘（上） */}
            <div className="w-full flex-none">
                <Clock
                    key={`opp-${clock[oppColor]}`}
                    label={t('opponent')}
                    dotClass={sideDotClass(oppColor)}
                    baseMs={clock[oppColor]}
                    running={phase === 'playing' && turn !== myColor}
                />
            </div>

            {/* 盤面 */}
            <div className={['relative flex min-h-0 w-full flex-1 items-center justify-center', shake ? 'animate-[shake_0.4s]' : ''].join(' ')}>
                {board}

                {phase === 'over' && result && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 rounded-lg bg-neutral-900/80 px-6 text-center backdrop-blur-sm">
                        <p className="text-2xl font-bold text-neutral-50">{resultTitle}</p>
                        <p className="text-sm text-neutral-300">{reasonLabel(result.reason)}</p>
                        <button
                            onClick={actions.backToLobby}
                            className="mt-2 flex items-center gap-2 rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
                        >
                            <RotateCcw className="h-4 w-4" />{t('playAgain')}
                        </button>
                    </div>
                )}
            </div>

            {/* 我方時鐘（下）＋ 控制 */}
            <div className="flex w-full flex-none items-center gap-3">
                <div className="flex-1">
                    <Clock
                        key={`me-${clock[myColor]}`}
                        label={`${t('you')}（${sideLabel(myColor)}）`}
                        dotClass={sideDotClass(myColor)}
                        baseMs={clock[myColor]}
                        running={myTurn}
                    />
                </div>
                <button
                    onClick={toggleMute}
                    aria-label={muted ? t('soundOn') : t('soundOff')}
                    className="rounded-lg border border-neutral-200 p-2 text-neutral-600 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800"
                >
                    {muted ? <VolumeX className="h-5 w-5" /> : <Volume2 className="h-5 w-5" />}
                </button>
                {phase === 'playing' && extraControls}
                {phase === 'playing' && (
                    <button
                        onClick={actions.resign}
                        className="flex items-center gap-2 rounded-lg border border-red-300 px-3 py-2 text-sm text-red-600 transition-colors hover:bg-red-50 dark:border-red-800 dark:hover:bg-red-950"
                    >
                        <Flag className="h-4 w-4" />{t('resign')}
                    </button>
                )}
            </div>

            {/* 狀態提示 */}
            <p className="flex-none text-sm text-neutral-500 dark:text-neutral-400">
                {phase === 'playing' && (myTurn ? t('yourTurn') : t('opponentTurn'))}
                {extraStatus}
            </p>
        </div>
    );
}
