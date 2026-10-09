"use client";

import { useId } from 'react';
import { useTranslations } from 'next-intl';
import { Crown, Shield, ThumbsDown, ThumbsUp } from 'lucide-react';
import { WoodDefs } from '../_shared/BoardWood';
import type { AvalonPhase, PlayerInfo, VoteResultData } from './avalon-types';

/// 座位怎麼點：組隊（隊長挑隊員）、刺殺（刺客挑目標）、不能點
export type SeatMode = 'team' | 'assassin' | null;

const TABLE_PX = 400; // 桌面 svg 的 viewBox 邊長（木紋濾鏡的頻率是以 user unit 計，太小紋路會消失）

export function AvalonTable({
    players, mySeat, leader, team, picked, target, votes, phase, round, results, sizes, failsRequired, rejects,
    mode, seatName, onSeat,
}: {
    players: PlayerInfo[];
    mySeat: number;
    leader: number | null;
    team: number[];      // 已提名（投票／任務中）的隊伍
    picked: number[];    // 隊長正在挑、還沒送出的隊員
    target: number | null;
    votes: VoteResultData | null;
    phase: AvalonPhase | undefined;
    round: number;
    results: boolean[];
    sizes: number[];
    failsRequired: number[];
    rejects: number;
    mode: SeatMode;
    seatName: (seat: number) => string;
    onSeat: (seat: number) => void;
}) {
    const t = useTranslations('Avalon');
    const uid = useId();
    const id = (name: string) => `${uid}-${name}`;
    const n = players.length;
    const assassinating = phase === 'assassinate';

    return (
        <div className="relative mx-auto aspect-square w-full max-w-[min(460px,58svh)]">
            {/* 桌面：木紋圓桌，刺殺階段整張壓暗泛紅 */}
            <div className={`absolute inset-[17%] overflow-hidden rounded-full shadow-xl shadow-amber-950/40 transition-shadow ${assassinating ? 'shadow-red-900/70' : ''}`}>
                <svg viewBox={`0 0 ${TABLE_PX} ${TABLE_PX}`} className="block h-full w-full" aria-hidden="true">
                    <defs><WoodDefs id={id} /></defs>
                    <rect width={TABLE_PX} height={TABLE_PX} fill={`url(#${id('wood')})`} />
                    <rect width={TABLE_PX} height={TABLE_PX} filter={`url(#${id('grain')})`} opacity={0.45} />
                    <rect width={TABLE_PX} height={TABLE_PX} fill={`url(#${id('vignette')})`} />
                    <circle cx={TABLE_PX / 2} cy={TABLE_PX / 2} r={TABLE_PX / 2 - 10} fill="none" stroke="#4a3418" strokeOpacity={0.35} strokeWidth={3} />
                    <rect width={TABLE_PX} height={TABLE_PX} fill="#2a1405" className="opacity-0 dark:opacity-[0.22]" />
                </svg>
                <div className={`absolute inset-0 bg-radial from-red-950/30 to-black/70 transition-opacity duration-700 ${assassinating ? 'opacity-100' : 'opacity-0'}`} />

                {/* 桌面中央：任務軌 + 否決軌 */}
                <div className="absolute inset-0 flex flex-col items-center justify-center gap-3">
                    <div className="flex gap-1.5 sm:gap-2">
                        {sizes.map((sz, i) => {
                            const done = i < results.length;
                            const success = done ? results[i] : null;
                            const current = !done && i === round && phase !== 'game_over';
                            const twoFails = (failsRequired[i] ?? 1) >= 2;
                            return (
                                <div key={i} title={twoFails ? t('twoFails') : undefined}
                                    aria-label={`${t('questN', { n: i + 1 })}：${sz}${success === true ? ` ${t('questSuccess')}` : success === false ? ` ${t('questFail')}` : ''}${twoFails ? `（${t('twoFails')}）` : ''}`}
                                    className={`relative flex size-8 items-center justify-center rounded-full border-2 text-sm font-bold shadow-md sm:size-10 sm:text-base ${success === true ? 'piece-flip border-primary-200 bg-primary-600 text-white'
                                        : success === false ? 'piece-flip border-red-300 bg-red-700 text-white'
                                            : current ? 'border-amber-200 bg-amber-100/90 text-amber-950 ring-2 ring-amber-300'
                                                : 'border-amber-900/40 bg-amber-50/70 text-amber-950/70'}`}>
                                    {sz}
                                    {twoFails && (
                                        <span className="absolute -right-1.5 -bottom-1.5 rounded-full bg-red-800 px-1 text-[10px] leading-4 font-bold text-white">×2</span>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                    {/* 否決軌：連續否決第 5 次壞人直接勝 */}
                    <div className="flex items-center gap-1.5" aria-label={t('rejects', { n: rejects })}>
                        {Array.from({ length: 5 }, (_, i) => (
                            <span key={i} className={`size-3 rounded-full border ${i < rejects
                                ? 'border-amber-200 bg-amber-500'
                                : i === 4 ? 'border-red-700 bg-red-900/40' : 'border-amber-900/50 bg-amber-950/20'}`} />
                        ))}
                    </div>
                </div>
            </div>

            {/* 座位：自己固定在正下方，其餘順時針排 */}
            {players.map(p => {
                const angle = (Math.PI / 2) + ((p.seat - mySeat) * 2 * Math.PI) / n;
                const left = 50 + 41 * Math.cos(angle);
                const top = 50 + 41 * Math.sin(angle);
                const me = p.seat === mySeat;
                const isLeader = leader === p.seat;
                const inTeam = team.includes(p.seat);
                const isPicked = picked.includes(p.seat);
                const isTarget = target === p.seat;
                const vote = votes?.votes.find(v => v.seat === p.seat);
                const selectable = mode === 'team' || (mode === 'assassin' && !me);
                const name = seatName(p.seat);
                const initial = p.name ? [...p.name][0] : String(p.seat + 1);
                const label = [
                    name, me ? `（${t('youTag')}）` : '',
                    isLeader ? ` ${t('seatLeader')}` : '', inTeam || isPicked ? ` ${t('seatInTeam')}` : '',
                    vote ? ` ${vote.approve ? t('approve') : t('reject')}` : '',
                ].join('');
                return (
                    <button key={p.seat} type="button" disabled={!selectable}
                        onClick={() => onSeat(p.seat)}
                        aria-pressed={selectable ? (mode === 'team' ? isPicked : isTarget) : undefined}
                        aria-label={label}
                        style={{ left: `${left}%`, top: `${top}%` }}
                        className={`group absolute flex w-20 -translate-x-1/2 -translate-y-1/2 flex-col items-center gap-1 ${selectable ? 'cursor-pointer' : 'cursor-default'}`}>
                        <span className={`relative flex size-11 items-center justify-center rounded-full border-2 text-lg font-bold shadow-md transition-transform sm:size-12 ${selectable ? 'group-hover:scale-110' : ''} ${me ? 'bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-100' : 'bg-neutral-100 text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200'} ${isTarget ? 'border-red-500 ring-4 ring-red-500/50'
                            : inTeam ? 'border-primary-500 ring-4 ring-primary-400/40'
                                : isPicked ? 'border-dashed border-primary-500'
                                    : 'border-neutral-300 dark:border-neutral-600'}`}>
                            {initial}
                            {isLeader && <Crown className="absolute -top-3.5 left-1/2 h-5 w-5 -translate-x-1/2 fill-amber-400 text-amber-600" />}
                            {(inTeam || isPicked) && (
                                <Shield className={`absolute -bottom-1.5 -left-1.5 h-5 w-5 fill-primary-500 text-primary-800 ${isPicked && !inTeam ? 'opacity-60' : ''}`} />
                            )}
                            {/* 投票牌：結果出來時全部同時翻開 */}
                            {vote && (
                                <span className={`piece-flip absolute -right-2.5 -bottom-1.5 flex size-6 items-center justify-center rounded-full border-2 border-white text-white shadow dark:border-neutral-900 ${vote.approve ? 'bg-green-600' : 'bg-red-600'}`}>
                                    {vote.approve ? <ThumbsUp className="h-3 w-3" /> : <ThumbsDown className="h-3 w-3" />}
                                </span>
                            )}
                        </span>
                        <span className={`max-w-full truncate rounded px-1 text-xs ${me ? 'font-bold text-primary-700 dark:text-primary-300' : 'text-neutral-700 dark:text-neutral-200'}`}>
                            {me ? t('youTag') : name}
                        </span>
                    </button>
                );
            })}
        </div>
    );
}
