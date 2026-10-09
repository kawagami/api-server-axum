"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { Swords, ThumbsUp, ThumbsDown, Volume2, VolumeX } from 'lucide-react';
import { sound } from '../_shared/sound';
import { AvalonChat } from './AvalonChat';
import { AvalonGameOver } from './AvalonGameOver';
import { AvalonQuestReveal } from './AvalonQuestReveal';
import { RoleIntro, RolePeek } from './AvalonRoleCard';
import { AvalonTable, type SeatMode } from './AvalonTable';
import { EVIL_ROLES, GOOD_ROLES, type QuestResultData, type RoleAssignedData } from './avalon-types';
import { useActionAlert } from './useActionAlert';
import type { UseAvalonRoom } from './useAvalonRoom';

export function AvalonPlay({ room }: { room: UseAvalonRoom }) {
    const t = useTranslations('Avalon');
    const tl = useTranslations('GameLobby');
    const { role, gamePhase, proposedTeam, voteResult, questResult, gameOver, chat, voted, cardPlayed, actions } = room;

    const phase = gamePhase?.phase;
    const mySeat = role?.your_seat ?? -1;
    const myRole = role?.your_role;
    const isGood = !!myRole && GOOD_ROLES.has(myRole);
    const isLeader = gamePhase?.leader === mySeat;
    const team = proposedTeam?.team ?? gamePhase?.team ?? [];
    const onTeam = team.includes(mySeat);
    const isAssassin = myRole === 'assassin';
    const seatName = (s: number) => role?.players.find(p => p.seat === s)?.name || t('playerN', { n: s + 1 });

    const [muted, setMuted] = useState(() => sound.isMuted());

    // 隊長挑的隊員／刺客挑的目標：階段、隊長或輪次換了就清空（render 中調整 state，不用 effect）
    const phaseKey = `${phase}-${gamePhase?.leader}-${gamePhase?.round}`;
    const [selKey, setSelKey] = useState(phaseKey);
    const [picked, setPicked] = useState<number[]>([]);
    const [target, setTarget] = useState<number | null>(null);
    if (selKey !== phaseKey) {
        setSelKey(phaseKey);
        setPicked([]);
        setTarget(null);
    }

    // 開局身分揭示：每次 role_assigned 看一次
    const [introDone, setIntroDone] = useState<RoleAssignedData | null>(null);
    // 任務卡揭曉：每個 quest_result 播一次；播的時候結局遮罩先等著
    const [revealDone, setRevealDone] = useState<QuestResultData | null>(null);
    const revealing = !!questResult && questResult !== revealDone && !!role;

    // 輪到自己行動的提醒
    const alertKind = phase === 'team_building' && isLeader ? 'leader'
        : phase === 'team_vote' && !voted ? 'vote'
            : phase === 'quest' && onTeam && !cardPlayed ? 'quest'
                : phase === 'assassinate' && isAssassin ? 'assassin' : null;
    useActionAlert(
        alertKind && gamePhase ? `${alertKind}-${gamePhase.round}-${gamePhase.leader}-${gamePhase.rejects}` : null,
        alertKind ? t(`alert_${alertKind}`) : null,
    );

    if (!role || !myRole) return null;

    const knownNames = role.known.map(seatName).join('、');
    const knownHint = myRole === 'merlin' ? t('known_merlin', { names: knownNames })
        : myRole === 'percival' ? t('known_percival', { names: knownNames })
            : EVIL_ROLES.has(myRole) && myRole !== 'oberon' ? t('known_evil', { names: knownNames || t('knownNoneInline') })
                : t('known_none');

    const mode: SeatMode = phase === 'team_building' && isLeader ? 'team'
        : phase === 'assassinate' && isAssassin ? 'assassin' : null;
    const onSeat = (s: number) => {
        sound.warmup();
        if (mode === 'team') {
            const size = gamePhase?.quest_size ?? 0;
            setPicked(prev => prev.includes(s) ? prev.filter(x => x !== s) : prev.length >= size ? prev : [...prev, s]);
        } else if (mode === 'assassin') {
            setTarget(s);
        }
    };

    const toggleMute = () => {
        const next = !muted;
        setMuted(next);
        sound.setMuted(next);
    };

    // 階段轉場橫幅文字
    const banner = !gamePhase || phase === 'game_over' ? null
        : phase === 'team_building' ? t('banner_team_building', { n: gamePhase.round + 1, name: seatName(gamePhase.leader) })
            : t(`banner_${phase}`);

    return (
        <div className="mx-auto flex h-[calc(100svh-120px)] w-full max-w-5xl flex-col gap-3 py-3 lg:flex-row">
            {/* 左：對局 */}
            <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto">
                <div className="flex items-center justify-between">
                    <p className="text-sm text-neutral-600 dark:text-neutral-300">
                        {gamePhase && phase !== 'game_over' && <>{t('questN', { n: gamePhase.round + 1 })} · {t('leaderIs', { name: seatName(gamePhase.leader) })}</>}
                    </p>
                    <button type="button" onClick={toggleMute} aria-label={muted ? tl('soundOn') : tl('soundOff')}
                        className="flex items-center rounded-lg border border-neutral-300 p-2 text-neutral-600 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800">
                        {muted ? <VolumeX className="h-5 w-5" /> : <Volume2 className="h-5 w-5" />}
                    </button>
                </div>

                <div className="relative">
                    <AvalonTable
                        players={role.players} mySeat={mySeat} leader={gamePhase?.leader ?? null}
                        team={phase === 'team_building' ? [] : team} picked={picked} target={target} votes={voteResult}
                        phase={phase} round={gamePhase?.round ?? 0} results={gamePhase?.results ?? []}
                        sizes={role.sizes} failsRequired={role.fails_required ?? []} rejects={gamePhase?.rejects ?? 0}
                        mode={mode} seatName={seatName} onSeat={onSeat} />
                    {/* 階段轉場：key 換了就重播一次淡入淡出 */}
                    {banner && (
                        <div key={phaseKey} aria-hidden="true"
                            className="avalon-banner pointer-events-none absolute inset-0 flex items-center justify-center">
                            <span className="rounded-lg bg-neutral-900/85 px-4 py-2 text-lg font-bold text-white shadow-xl">{banner}</span>
                        </div>
                    )}
                </div>

                <div className="flex flex-col gap-3 sm:flex-row">
                    <RolePeek role={myRole} knownHint={knownHint} />
                    <div className="flex flex-1 flex-col gap-3">
                        <ActionPanel room={room} picked={picked} target={target} onPropose={() => actions.proposeTeam(picked)} seatName={seatName} />
                        {(voteResult || questResult) && (
                            <div className="rounded-lg border border-neutral-200 p-3 text-sm text-neutral-600 dark:border-neutral-700 dark:text-neutral-300">
                                {voteResult && <p>{voteResult.approved ? t('teamApproved') : t('teamRejected')}</p>}
                                {questResult && !revealing && (
                                    <p>{t('questOutcome', { round: questResult.round + 1, result: questResult.success ? t('questSuccess') : t('questFail'), fails: questResult.fails })}</p>
                                )}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* 右：聊天 */}
            <div className="flex h-56 flex-col lg:h-auto lg:w-80">
                <AvalonChat chat={chat} onSend={actions.sendChat} />
            </div>

            {introDone !== role && <RoleIntro role={myRole} knownHint={knownHint} onDone={() => setIntroDone(role)} />}

            {revealing && questResult && (
                <AvalonQuestReveal key={questResult.round} round={questResult.round} size={role.sizes[questResult.round] ?? 0}
                    fails={questResult.fails} success={questResult.success} onDone={() => setRevealDone(questResult)} />
            )}

            {gameOver && !revealing && (
                <AvalonGameOver gameOver={gameOver} isGood={isGood} mySeat={mySeat} seatName={seatName} onBack={actions.backToLobby} />
            )}
        </div>
    );
}

function ActionPanel({ room, picked, target, onPropose, seatName }: {
    room: UseAvalonRoom; picked: number[]; target: number | null; onPropose: () => void; seatName: (s: number) => string;
}) {
    const t = useTranslations('Avalon');
    const { role, gamePhase, proposedTeam, voted, cardPlayed, actions } = room;

    if (!role || !gamePhase) return null;
    const phase = gamePhase.phase;
    const mySeat = role.your_seat;
    const isGood = GOOD_ROLES.has(role.your_role);
    const isLeader = gamePhase.leader === mySeat;
    const team = proposedTeam?.team ?? gamePhase.team ?? [];
    const onTeam = team.includes(mySeat);
    const isAssassin = role.your_role === 'assassin';

    return (
        <div className="rounded-lg border border-neutral-200 p-3 dark:border-neutral-700">
            {phase === 'team_building' && (isLeader ? (
                <div className="flex flex-col gap-2">
                    <p className="text-sm font-medium">{t('pickOnTable', { n: picked.length, size: gamePhase.quest_size })}</p>
                    <button onClick={onPropose} disabled={picked.length !== gamePhase.quest_size}
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
                    <p className="text-sm font-medium text-red-600 dark:text-red-400">
                        {t('assassinPrompt')} · {target !== null ? seatName(target) : t('assassinPickOnTable')}
                    </p>
                    <button onClick={() => target !== null && actions.assassinate(target)} disabled={target === null}
                        className="flex items-center gap-1.5 self-start rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700 disabled:opacity-40">
                        <Swords className="h-4 w-4" />{t('confirmAssassinate')}
                    </button>
                </div>
            ) : <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('assassinThinking')}</p>)}
        </div>
    );
}
