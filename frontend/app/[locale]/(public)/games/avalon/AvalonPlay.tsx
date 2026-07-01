"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { Crown, Check, X, Swords, Skull, ThumbsUp, ThumbsDown, RotateCcw } from 'lucide-react';
import { AvalonChat } from './AvalonChat';
import { EVIL_ROLES, GOOD_ROLES } from './avalon-types';
import type { UseAvalonRoom } from './useAvalonRoom';

export function AvalonPlay({ room }: { room: UseAvalonRoom }) {
    const t = useTranslations('Avalon');
    const { role, gamePhase, proposedTeam, voteResult, questResult, gameOver, chat, actions } = room;

    const phase = gamePhase?.phase;

    if (!role) return null;
    const mySeat = role.your_seat;
    const myRole = role.your_role;
    const isGood = GOOD_ROLES.has(myRole);
    const seatName = (s: number) => role.players.find(p => p.seat === s)?.name || t('playerN', { n: s + 1 });

    const team = proposedTeam?.team ?? gamePhase?.team ?? [];

    // 角色提示
    const knownNames = role.known.map(seatName).join('、');
    const knownHint = myRole === 'merlin' ? t('known_merlin', { names: knownNames })
        : myRole === 'percival' ? t('known_percival', { names: knownNames })
            : EVIL_ROLES.has(myRole) && myRole !== 'oberon' ? t('known_evil', { names: knownNames || t('knownNoneInline') })
                : t('known_none');

    return (
        <div className="mx-auto flex h-[calc(100svh-120px)] w-full max-w-5xl flex-col gap-3 py-3 lg:flex-row">
            {/* 左：對局 */}
            <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto">
                {/* 角色橫幅（只用自己的 role_assigned） */}
                <div className={`rounded-lg border-l-4 p-3 ${isGood ? 'border-primary-500 bg-primary-50 dark:bg-primary-950' : 'border-red-500 bg-red-50 dark:bg-red-950'}`}>
                    <p className="font-bold text-neutral-800 dark:text-neutral-100">
                        {t('youAre')}：{t(`role_${myRole}`)}
                        <span className={`ml-2 text-xs ${isGood ? 'text-primary-600' : 'text-red-600'}`}>（{isGood ? t('sideGood') : t('sideEvil')}）</span>
                    </p>
                    <p className="mt-1 text-sm text-neutral-600 dark:text-neutral-300">{knownHint}</p>
                </div>

                {/* 任務軌 */}
                <div className="flex items-center justify-between rounded-lg border border-neutral-200 px-3 py-2 dark:border-neutral-700">
                    <div className="flex gap-2">
                        {role.sizes.map((sz, i) => {
                            const done = gamePhase && i < gamePhase.results.length;
                            const success = done ? gamePhase!.results[i] : null;
                            const current = gamePhase?.round === i && phase !== 'game_over';
                            return (
                                <div key={i} className={`flex h-9 w-9 items-center justify-center rounded-full border-2 text-sm font-bold ${success === true ? 'border-green-500 bg-green-500 text-white'
                                    : success === false ? 'border-red-500 bg-red-500 text-white'
                                        : current ? 'border-primary-500 text-primary-600 dark:text-primary-300'
                                            : 'border-neutral-300 text-neutral-400 dark:border-neutral-600'}`}>
                                    {sz}
                                </div>
                            );
                        })}
                    </div>
                    <div className="text-right text-xs text-neutral-500 dark:text-neutral-400">
                        <div>{t('rejects', { n: gamePhase?.rejects ?? 0 })}</div>
                        {gamePhase && phase !== 'game_over' && <div>{t('leaderIs', { name: seatName(gamePhase.leader) })}</div>}
                    </div>
                </div>

                {/* 玩家列 */}
                <div className="flex flex-wrap gap-2">
                    {role.players.map(p => {
                        const leader = gamePhase?.leader === p.seat;
                        const inTeam = team.includes(p.seat);
                        const me = p.seat === mySeat;
                        return (
                            <span key={p.seat} className={`flex items-center gap-1 rounded-full border px-2.5 py-1 text-sm ${inTeam ? 'border-primary-500 bg-primary-50 dark:bg-primary-950' : 'border-neutral-200 dark:border-neutral-700'}`}>
                                {leader && <Crown className="h-3.5 w-3.5 text-amber-500" />}
                                <span className={me ? 'font-bold text-primary-700 dark:text-primary-300' : 'text-neutral-700 dark:text-neutral-200'}>
                                    {seatName(p.seat)}{me && `（${t('youTag')}）`}
                                </span>
                                {inTeam && <Check className="h-3.5 w-3.5 text-primary-600" />}
                            </span>
                        );
                    })}
                </div>

                {/* 階段面板（key 隨階段/隊長/輪次重掛載，重置本地選擇） */}
                <ActionPanel key={`${phase}-${gamePhase?.leader}-${gamePhase?.round}`} room={room} seatName={seatName} />

                {/* 投票 / 任務結果 */}
                {voteResult && (
                    <div className="rounded-lg border border-neutral-200 p-3 text-sm dark:border-neutral-700">
                        <p className="font-medium">{voteResult.approved ? t('teamApproved') : t('teamRejected')}</p>
                        <div className="mt-1 flex flex-wrap gap-2">
                            {voteResult.votes.map(v => (
                                <span key={v.seat} className="flex items-center gap-1 text-xs">
                                    {v.approve ? <Check className="h-3.5 w-3.5 text-green-600" /> : <X className="h-3.5 w-3.5 text-red-600" />}
                                    {seatName(v.seat)}
                                </span>
                            ))}
                        </div>
                    </div>
                )}
                {questResult && (
                    <div className="rounded-lg border border-neutral-200 p-3 text-sm dark:border-neutral-700">
                        {t('questOutcome', { round: questResult.round + 1, result: questResult.success ? t('questSuccess') : t('questFail'), fails: questResult.fails })}
                    </div>
                )}
            </div>

            {/* 右：聊天 */}
            <div className="flex h-56 flex-col lg:h-auto lg:w-80">
                <AvalonChat chat={chat} onSend={actions.sendChat} />
            </div>

            {/* 結局遮罩 */}
            {gameOver && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-neutral-900/80 p-4 backdrop-blur-sm">
                    <div className="flex w-full max-w-md flex-col gap-3 rounded-lg bg-white p-5 dark:bg-neutral-800">
                        <h2 className="text-center text-2xl font-bold text-neutral-800 dark:text-neutral-100">
                            {gameOver.winner === 'good' ? t('goodWins') : gameOver.winner === 'evil' ? t('evilWins') : t('draw')}
                        </h2>
                        <p className="text-center text-sm text-neutral-500 dark:text-neutral-400">{t(`reason_${gameOver.reason}`)}</p>
                        <ul className="flex flex-col gap-1 text-sm">
                            {gameOver.roles.map(r => (
                                <li key={r.seat} className="flex justify-between rounded px-2 py-1 odd:bg-neutral-100 dark:odd:bg-neutral-700">
                                    <span>{seatName(r.seat)}</span>
                                    <span className={EVIL_ROLES.has(r.role) ? 'text-red-600 dark:text-red-400' : 'text-primary-600 dark:text-primary-300'}>{t(`role_${r.role}`)}</span>
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

function ActionPanel({ room, seatName }: { room: UseAvalonRoom; seatName: (s: number) => string }) {
    const t = useTranslations('Avalon');
    const { role, gamePhase, proposedTeam, voted, cardPlayed, actions } = room;
    const [picked, setPicked] = useState<number[]>([]);
    const [target, setTarget] = useState<number | null>(null);

    if (!role || !gamePhase) return null;
    const phase = gamePhase.phase;
    const mySeat = role.your_seat;
    const isGood = GOOD_ROLES.has(role.your_role);
    const isLeader = gamePhase.leader === mySeat;
    const team = proposedTeam?.team ?? gamePhase.team ?? [];
    const onTeam = team.includes(mySeat);
    const isAssassin = role.your_role === 'assassin';

    const togglePick = (s: number) => setPicked(prev =>
        prev.includes(s) ? prev.filter(x => x !== s)
            : prev.length >= gamePhase.quest_size ? prev : [...prev, s]);

    return (
        <div className="rounded-lg border border-neutral-200 p-3 dark:border-neutral-700">
            {phase === 'team_building' && (isLeader ? (
                <div className="flex flex-col gap-2">
                    <p className="text-sm font-medium">{t('youAreLeader', { size: gamePhase.quest_size })}</p>
                    <div className="flex flex-wrap gap-2">
                        {role.players.map(p => (
                            <button key={p.seat} onClick={() => togglePick(p.seat)}
                                className={`rounded-full border px-3 py-1 text-sm transition-colors ${picked.includes(p.seat) ? 'border-primary-500 bg-primary-600 text-white' : 'border-neutral-300 hover:bg-neutral-100 dark:border-neutral-600 dark:hover:bg-neutral-800'}`}>
                                {seatName(p.seat)}
                            </button>
                        ))}
                    </div>
                    <button onClick={() => actions.proposeTeam(picked)} disabled={picked.length !== gamePhase.quest_size}
                        className="self-start rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:opacity-40">
                        {t('proposeTeam')}（{picked.length}/{gamePhase.quest_size}）
                    </button>
                </div>
            ) : <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('waitLeader', { name: seatName(gamePhase.leader) })}</p>)}

            {phase === 'team_vote' && (
                <div className="flex flex-col gap-2">
                    <p className="text-sm">{t('voteOnTeam')}：{team.map(seatName).join('、')}</p>
                    {voted ? <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('votedWait')}</p> : (
                        <div className="flex gap-2">
                            <button onClick={() => actions.teamVote(true)} className="flex items-center gap-1.5 rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700">
                                <ThumbsUp className="h-4 w-4" />{t('approve')}
                            </button>
                            <button onClick={() => actions.teamVote(false)} className="flex items-center gap-1.5 rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700">
                                <ThumbsDown className="h-4 w-4" />{t('reject')}
                            </button>
                        </div>
                    )}
                </div>
            )}

            {phase === 'quest' && (
                <div className="flex flex-col gap-2">
                    <p className="text-sm">{t('questTeam')}：{team.map(seatName).join('、')}</p>
                    {!onTeam ? <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('notOnQuest')}</p>
                        : cardPlayed ? <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('cardPlayedWait')}</p> : (
                            <div className="flex gap-2">
                                <button onClick={() => actions.questCard(true)} className="rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700">{t('questSuccess')}</button>
                                {!isGood && <button onClick={() => actions.questCard(false)} className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700">{t('questFail')}</button>}
                            </div>
                        )}
                </div>
            )}

            {phase === 'assassinate' && (isAssassin ? (
                <div className="flex flex-col gap-2">
                    <p className="flex items-center gap-1.5 text-sm font-medium text-red-600"><Skull className="h-4 w-4" />{t('assassinPrompt')}</p>
                    <div className="flex flex-wrap gap-2">
                        {role.players.filter(p => p.seat !== mySeat).map(p => (
                            <button key={p.seat} onClick={() => setTarget(p.seat)}
                                className={`rounded-full border px-3 py-1 text-sm transition-colors ${target === p.seat ? 'border-red-500 bg-red-600 text-white' : 'border-neutral-300 hover:bg-neutral-100 dark:border-neutral-600 dark:hover:bg-neutral-800'}`}>
                                {seatName(p.seat)}
                            </button>
                        ))}
                    </div>
                    <button onClick={() => target !== null && actions.assassinate(target)} disabled={target === null}
                        className="flex items-center gap-1.5 self-start rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700 disabled:opacity-40">
                        <Swords className="h-4 w-4" />{t('confirmAssassinate')}
                    </button>
                </div>
            ) : <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('assassinThinking')}</p>)}
        </div>
    );
}
