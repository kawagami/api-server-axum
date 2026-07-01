"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { RotateCcw, Trophy } from 'lucide-react';
import {
    ACCUMULATION_ACTIONS, COMPOUND_ACTIONS, HARVEST_ROUNDS,
    type FarmState, type PlayerState,
} from './farm-types';
import type { UseFarmRoom } from './useFarmRoom';

const RES_KEYS = ['wood', 'clay', 'reed', 'stone', 'grain', 'veg', 'food'] as const;
const ANIMAL_KEYS = ['sheep', 'boar', 'cattle'] as const;

export function FarmPlay({ farm }: { farm: UseFarmRoom }) {
    const t = useTranslations('Farm');
    const { state, room, mySeat, gameOver, actions } = farm;
    if (!state) return null;

    const seatName = (s: number) => room?.players.find(p => p.seat === s)?.name || t('playerN', { n: s + 1 });
    const myTurn = state.phase === 'placing' && state.current_player === mySeat;

    return (
        <div className="mx-auto flex h-[calc(100svh-120px)] w-full max-w-5xl flex-col gap-3 overflow-y-auto py-3">
            {/* 回合資訊 */}
            <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-neutral-200 px-3 py-2 dark:border-neutral-700">
                <span className="flex items-center gap-2 text-sm font-medium">
                    {t('round', { n: state.round })}
                    {HARVEST_ROUNDS.has(state.round) && <span className="rounded-full bg-amber-100 px-2 py-0.5 text-xs text-amber-700 dark:bg-amber-900 dark:text-amber-200">{t('harvestRound')}</span>}
                </span>
                <span className="text-sm text-neutral-600 dark:text-neutral-300">
                    {state.current_player === null ? t('settling')
                        : myTurn ? <span className="font-semibold text-primary-600">{t('yourTurn')}</span>
                            : t('currentTurn', { name: seatName(state.current_player) })}
                </span>
            </div>

            {/* 行動板 */}
            <ActionBoard key={`${state.round}-${state.current_player}`} state={state} myTurn={myTurn} onAction={actions.sendAction} />

            {/* 各家農場 */}
            <div className="grid gap-3 sm:grid-cols-2">
                {state.players.map((p, seat) => (
                    <PlayerCard key={seat} p={p} name={seatName(seat)}
                        isMe={seat === mySeat} isCurrent={seat === state.current_player} />
                ))}
            </div>

            {/* 結算 */}
            {gameOver && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-neutral-900/80 p-4 backdrop-blur-sm">
                    <div className="flex w-full max-w-md flex-col gap-3 rounded-lg bg-white p-5 dark:bg-neutral-800">
                        <h2 className="flex items-center justify-center gap-2 text-2xl font-bold text-neutral-800 dark:text-neutral-100">
                            <Trophy className="h-6 w-6 text-amber-500" />{t('gameOverTitle')}
                        </h2>
                        <ul className="flex flex-col gap-1 text-sm">
                            {gameOver.scores
                                .map((score, seat) => ({ score, seat }))
                                .sort((a, b) => b.score - a.score)
                                .map((e, i) => (
                                    <li key={e.seat} className="flex justify-between rounded px-2 py-1 odd:bg-neutral-100 dark:odd:bg-neutral-700">
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

function ActionBoard({ state, myTurn, onAction }: { state: FarmState; myTurn: boolean; onAction: (a: string, input?: Record<string, unknown>) => void; }) {
    const t = useTranslations('Farm');
    const [compound, setCompound] = useState<string | null>(null);
    const [form, setForm] = useState<Record<string, number | boolean>>({});

    const amountOf = (a: string) => state.accumulation.find(x => x.action === a)?.amount;
    const num = (k: string) => Number(form[k] ?? 0);

    const click = (a: string) => {
        if (!myTurn) return;
        if (COMPOUND_ACTIONS.has(a)) { setCompound(a); setForm({}); return; }
        onAction(a);
    };

    const confirmCompound = () => {
        if (!compound) return;
        let input: Record<string, unknown> = {};
        if (compound === 'sow') input = { grain_fields: num('grain_fields'), veg_fields: num('veg_fields') };
        else if (compound === 'build_rooms') input = { rooms: num('rooms'), stables: num('stables') };
        else if (compound === 'fences') input = { pasture_tiles: num('pasture_tiles'), pasture_stable: !!form.pasture_stable };
        onAction(compound, input);
        setCompound(null);
    };

    return (
        <div className="rounded-lg border border-neutral-200 p-3 dark:border-neutral-700">
            <h2 className="mb-2 text-sm font-semibold text-neutral-600 dark:text-neutral-300">{t('actionsHeading')}</h2>
            <div className="flex flex-wrap gap-2">
                {state.available_actions.map(a => {
                    const amt = amountOf(a);
                    return (
                        <button key={a} onClick={() => click(a)} disabled={!myTurn}
                            className={`flex flex-col items-start gap-0.5 rounded-lg border px-3 py-1.5 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${compound === a ? 'border-primary-500 bg-primary-600 text-white' : 'border-neutral-300 hover:bg-neutral-100 dark:border-neutral-600 dark:hover:bg-neutral-800'}`}>
                            <span className="flex items-center gap-1 text-sm">
                                {t(`action_${a}`)}
                                {ACCUMULATION_ACTIONS.has(a) && amt !== undefined && (
                                    <span className="rounded-full bg-amber-100 px-1.5 text-xs text-amber-700 dark:bg-amber-900 dark:text-amber-200">{amt}</span>
                                )}
                            </span>
                            <span className="text-[11px] leading-tight opacity-70">{t(`hint_${a}`)}</span>
                        </button>
                    );
                })}
            </div>

            {/* 複合動作 input 表單 */}
            {compound && myTurn && (
                <div className="mt-3 flex flex-col gap-2 rounded-lg bg-neutral-50 p-3 dark:bg-neutral-800">
                    <p className="text-sm font-medium">{t(`action_${compound}`)}</p>
                    <div className="flex flex-wrap gap-3 text-sm">
                        {compound === 'sow' && (<>
                            <NumIn label={t('inGrainFields')} v={num('grain_fields')} on={v => setForm(f => ({ ...f, grain_fields: v }))} />
                            <NumIn label={t('inVegFields')} v={num('veg_fields')} on={v => setForm(f => ({ ...f, veg_fields: v }))} />
                        </>)}
                        {compound === 'build_rooms' && (<>
                            <NumIn label={t('inRooms')} v={num('rooms')} on={v => setForm(f => ({ ...f, rooms: v }))} />
                            <NumIn label={t('inStables')} v={num('stables')} on={v => setForm(f => ({ ...f, stables: v }))} />
                        </>)}
                        {compound === 'fences' && (<>
                            <NumIn label={t('inPastureTiles')} v={num('pasture_tiles')} on={v => setForm(f => ({ ...f, pasture_tiles: v }))} />
                            <label className="flex items-center gap-1.5">
                                <input type="checkbox" checked={!!form.pasture_stable} onChange={e => setForm(f => ({ ...f, pasture_stable: e.target.checked }))} />
                                {t('inPastureStable')}
                            </label>
                        </>)}
                    </div>
                    <div className="flex gap-2">
                        <button onClick={confirmCompound} className="rounded-lg bg-primary-600 px-4 py-1.5 text-sm font-medium text-white hover:bg-primary-700">{t('confirm')}</button>
                        <button onClick={() => setCompound(null)} className="rounded-lg border border-neutral-300 px-4 py-1.5 text-sm dark:border-neutral-600">{t('cancel')}</button>
                    </div>
                </div>
            )}
        </div>
    );
}

function NumIn({ label, v, on }: { label: string; v: number; on: (v: number) => void }) {
    return (
        <label className="flex items-center gap-1.5">
            {label}
            <input type="number" min={0} value={v} onChange={e => on(Math.max(0, Number(e.target.value) || 0))}
                className="w-16 rounded border border-neutral-300 bg-white px-2 py-1 dark:border-neutral-600 dark:bg-neutral-900" />
        </label>
    );
}

function PlayerCard({ p, name, isMe, isCurrent }: { p: PlayerState; name: string; isMe: boolean; isCurrent: boolean }) {
    const t = useTranslations('Farm');
    const grain = p.fields.reduce((s, f) => s + (f?.crop === 'grain' ? f.count : 0), 0);
    const veg = p.fields.reduce((s, f) => s + (f?.crop === 'veg' ? f.count : 0), 0);
    const emptyFields = p.fields.filter(f => f === null).length;

    return (
        <div className={`flex flex-col gap-1.5 rounded-lg border p-3 text-sm ${isCurrent ? 'border-primary-500 ring-1 ring-primary-400' : 'border-neutral-200 dark:border-neutral-700'}`}>
            <div className="flex items-center justify-between">
                <span className="font-semibold text-neutral-800 dark:text-neutral-100">{name}{isMe && <span className="ml-1 text-xs text-primary-600">（{t('youTag')}）</span>}</span>
                {p.begging > 0 && <span className="text-xs font-semibold text-red-600">{t('begging', { n: p.begging })}</span>}
            </div>

            <div className="text-neutral-600 dark:text-neutral-300">
                {t(`house_${p.house}`)} ×{p.rooms} · {t('familyN', { n: p.family })} · {t('freeTilesN', { n: p.free_tiles })}
                {p.loose_stables > 0 && ` · ${t('looseStablesN', { n: p.loose_stables })}`}
            </div>

            <div className="text-neutral-600 dark:text-neutral-300">
                {t('fieldsLabel')}: {t('crop_grain')}{grain} / {t('crop_veg')}{veg} / {t('emptyField')}{emptyFields}
            </div>

            <div className="text-neutral-600 dark:text-neutral-300">
                {t('pasturesLabel')}: {p.pastures.length === 0 ? '—' : p.pastures.map((pa, i) =>
                    `${t('tilesN', { n: pa.tiles })}${pa.stable ? `[${t('stableTag')}]` : ''}${pa.animal ? ` ${t(`animal_${pa.animal.kind}`)}×${pa.animal.count}` : ''}`
                ).join('；')}
            </div>

            <div className="grid grid-cols-4 gap-1 text-xs text-neutral-500 dark:text-neutral-400">
                {RES_KEYS.map(k => <span key={k}>{t(`res_${k}`)} {p[k]}</span>)}
            </div>
            <div className="flex gap-3 text-xs text-neutral-500 dark:text-neutral-400">
                {ANIMAL_KEYS.map(k => <span key={k}>{t(`animal_${k}`)} {p[k]}</span>)}
            </div>
        </div>
    );
}
