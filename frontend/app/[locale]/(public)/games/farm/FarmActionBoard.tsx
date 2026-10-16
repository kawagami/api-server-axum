"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { Lock, Minus, Plus } from 'lucide-react';
import Modal from '@/components/modal';
import { cn } from '@/libs/cn';
import { ACCUMULATION_ACTIONS, COMPOUND_ACTIONS, type FarmState, type PlayerState } from './farm-types';
import { blockedReason, compoundCost, compoundError, compoundMax } from './farm-rules';
import { ACCUM_RES, ACTION_GROUPS, ACTION_ICON, RES_ICON, seatColor } from './farm-ui';

/// 行動板：依類別分組顯示全部 18 格（可選 / 已被誰佔 / 尚未開放 / 我目前做不了的原因）。
/// 複合動作（播種／蓋房／圍牧場）開對話框，步進器有上限、即時算花費，不夠就鎖確認鈕。
export function FarmActionBoard({
    state, me, myTurn, seatName, onAction, onCompoundOpen,
}: {
    state: FarmState;
    me: PlayerState | null;
    myTurn: boolean;
    seatName: (s: number) => string;
    onAction: (a: string, input?: Record<string, unknown>) => void;
    onCompoundOpen: (a: string) => void;
}) {
    const t = useTranslations('Farm');
    const [compound, setCompound] = useState<string | null>(null);

    const available = new Set(state.available_actions);
    const occupiedBy = new Map(state.occupied.map(o => [o.action, o.seat]));
    const amountOf = (a: string) => state.accumulation.find(x => x.action === a)?.amount ?? 0;

    const click = (a: string) => {
        if (!myTurn) return;
        if (COMPOUND_ACTIONS.has(a)) { setCompound(a); onCompoundOpen(a); return; }
        onAction(a);
    };

    return (
        <section className="flex flex-col gap-3 rounded-lg border border-neutral-200 p-3 dark:border-neutral-700">
            <h2 className="text-sm font-semibold text-neutral-600 dark:text-neutral-300">{t('actionsHeading')}</h2>
            {ACTION_GROUPS.map(g => (
                <div key={g.key} className="flex flex-col gap-1.5">
                    <h3 className="text-xs font-medium text-neutral-500 dark:text-neutral-400">{t(`group_${g.key}`)}</h3>
                    <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-3">
                        {g.actions.map(a => {
                            const seat = occupiedBy.get(a);
                            const open = available.has(a);
                            const locked = !open && seat === undefined;
                            const blocked = open && me ? blockedReason(a, me) : null;
                            return (
                                <ActionCard key={a} action={a} amount={amountOf(a)} occupiedSeat={seat} occupiedName={seat !== undefined ? seatName(seat) : ''}
                                    locked={locked} blocked={blocked} clickable={myTurn && open && !blocked} onClick={() => click(a)} />
                            );
                        })}
                    </div>
                </div>
            ))}

            {compound && myTurn && me && (
                <CompoundDialog action={compound} me={me}
                    onClose={() => setCompound(null)}
                    onConfirm={input => { onAction(compound, input); setCompound(null); }} />
            )}
        </section>
    );
}

function ActionCard({
    action, amount, occupiedSeat, occupiedName, locked, blocked, clickable, onClick,
}: {
    action: string; amount: number; occupiedSeat: number | undefined; occupiedName: string;
    locked: boolean; blocked: string | null; clickable: boolean; onClick: () => void;
}) {
    const t = useTranslations('Farm');
    const accum = ACCUMULATION_ACTIONS.has(action);
    return (
        <button type="button" onClick={onClick} disabled={!clickable}
            className={cn('relative flex min-h-16 items-start gap-2 rounded-lg border px-2 py-1.5 text-left transition-colors',
                clickable ? 'border-primary-300 bg-primary-50 hover:bg-primary-100 dark:border-primary-700 dark:bg-primary-950 dark:hover:bg-primary-900'
                    : 'cursor-not-allowed border-neutral-200 dark:border-neutral-700',
                (locked || occupiedSeat !== undefined) && 'opacity-50')}>
            <span className="text-2xl leading-none" aria-hidden="true">{ACTION_ICON[action]}</span>
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className="text-sm font-medium text-neutral-800 dark:text-neutral-100">{t(`action_${action}`)}</span>
                <span className="text-[11px] leading-tight text-neutral-500 dark:text-neutral-400">{t(`hint_${action}`)}</span>
                {locked && <span className="flex items-center gap-1 text-[11px] text-neutral-500"><Lock className="h-3 w-3" />{t('locked')}</span>}
                {occupiedSeat !== undefined && (
                    <span className={cn('flex items-center gap-1 text-[11px] font-medium', seatColor(occupiedSeat).text)}>
                        <span className={cn('h-2 w-2 rounded-full', seatColor(occupiedSeat).dot)} />{occupiedName}
                    </span>
                )}
                {blocked && <span className="text-[11px] text-red-600 dark:text-red-400">{t(blocked)}</span>}
            </span>
            {accum && !locked && amount > 0 && (
                <span className="absolute right-1 top-1 rounded-full bg-amber-100 px-1.5 text-xs font-bold text-amber-800 dark:bg-amber-900 dark:text-amber-100">
                    {amount}{RES_ICON[ACCUM_RES[action]]}
                </span>
            )}
        </button>
    );
}

const FIELDS: Record<string, string[]> = {
    sow: ['grain_fields', 'veg_fields'],
    build_rooms: ['rooms', 'stables'],
    fences: ['pasture_tiles'],
};
const FIELD_LABEL: Record<string, string> = {
    grain_fields: 'inGrainFields', veg_fields: 'inVegFields', rooms: 'inRooms', stables: 'inStables', pasture_tiles: 'inPastureTiles',
};

function CompoundDialog({ action, me, onClose, onConfirm }: {
    action: string; me: PlayerState; onClose: () => void; onConfirm: (input: Record<string, unknown>) => void;
}) {
    const t = useTranslations('Farm');
    const [form, setForm] = useState<Record<string, number | boolean>>(action === 'fences' ? { pasture_tiles: 1 } : {});
    const num = (k: string) => Number(form[k] ?? 0);
    const cost = compoundCost(action, me, form);
    const error = compoundError(action, me, form);

    const confirm = () => {
        if (error) return;
        const input: Record<string, unknown> = {};
        for (const k of FIELDS[action] ?? []) input[k] = num(k);
        if (action === 'fences') input.pasture_stable = !!form.pasture_stable;
        onConfirm(input);
    };

    return (
        <Modal label={t(`action_${action}`)} onClose={onClose} size="md" surface="public" className="flex flex-col gap-4 p-5">
            <h2 className="flex items-center gap-2 text-lg font-bold text-neutral-800 dark:text-neutral-100">
                <span aria-hidden="true">{ACTION_ICON[action]}</span>{t(`action_${action}`)}
            </h2>
            <p className="text-sm text-neutral-600 dark:text-neutral-300">{t(`hint_${action}`)}</p>

            <div className="flex flex-col gap-2">
                {(FIELDS[action] ?? []).map(k => (
                    <Stepper key={k} label={t(FIELD_LABEL[k])} value={num(k)} min={action === 'fences' ? 1 : 0}
                        max={compoundMax(action, k, me)} onChange={v => setForm(f => ({ ...f, [k]: v }))} />
                ))}
                {action === 'fences' && (
                    <label className="flex items-center gap-2 text-sm">
                        <input type="checkbox" checked={!!form.pasture_stable} onChange={e => setForm(f => ({ ...f, pasture_stable: e.target.checked }))} />
                        {t('inPastureStable')}
                    </label>
                )}
            </div>

            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 rounded-md bg-neutral-100 px-3 py-2 text-sm dark:bg-neutral-700">
                <span className="font-medium text-neutral-600 dark:text-neutral-300">{t('costLabel')}</span>
                {Object.entries(cost).filter(([, v]) => (v ?? 0) > 0).map(([k, v]) => {
                    const have = me[k as keyof typeof cost];
                    return (
                        <span key={k} className={cn((v ?? 0) > have ? 'font-semibold text-red-600 dark:text-red-400' : 'text-neutral-700 dark:text-neutral-200')}>
                            {RES_ICON[k as keyof typeof cost]} {v}<span className="text-xs opacity-70"> / {have}</span>
                        </span>
                    );
                })}
                {Object.values(cost).every(v => !v) && <span className="text-neutral-500">—</span>}
            </div>

            {error && <p className="text-sm text-red-600 dark:text-red-400">{t(error)}</p>}

            <div className="flex justify-end gap-2">
                <button type="button" onClick={onClose} className="rounded-lg border border-neutral-300 px-4 py-1.5 text-sm transition-colors hover:bg-neutral-100 dark:border-neutral-600 dark:hover:bg-neutral-700">{t('cancel')}</button>
                <button type="button" onClick={confirm} disabled={!!error}
                    className="rounded-lg bg-primary-600 px-4 py-1.5 text-sm font-medium text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-40">
                    {t('confirm')}
                </button>
            </div>
        </Modal>
    );
}

function Stepper({ label, value, min, max, onChange }: { label: string; value: number; min: number; max: number; onChange: (v: number) => void }) {
    const btn = 'flex h-8 w-8 items-center justify-center rounded-lg border border-neutral-300 transition-colors hover:bg-neutral-100 disabled:cursor-not-allowed disabled:opacity-30 dark:border-neutral-600 dark:hover:bg-neutral-700';
    return (
        <div className="flex items-center justify-between gap-3 text-sm">
            <span>{label}</span>
            <span className="flex items-center gap-2">
                <button type="button" className={btn} onClick={() => onChange(Math.max(min, value - 1))} disabled={value <= min} aria-label={`${label} -1`}><Minus className="h-4 w-4" /></button>
                <span className="w-6 text-center font-mono text-base font-semibold" aria-live="polite">{value}</span>
                <button type="button" className={btn} onClick={() => onChange(Math.min(max, value + 1))} disabled={value >= max} aria-label={`${label} +1`}><Plus className="h-4 w-4" /></button>
                <span className="w-10 text-xs text-neutral-500">≤ {max}</span>
            </span>
        </div>
    );
}
