"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import {
    EyeOff, Ghost, Moon, Shield, ShieldCheck, Skull, Swords, WandSparkles, Fingerprint,
    type LucideIcon,
} from 'lucide-react';
import Modal from '@/components/modal';
import { sound } from '../_shared/sound';
import { EVIL_ROLES, type AvalonRole } from './avalon-types';

export const ROLE_ICON: Record<AvalonRole, LucideIcon> = {
    merlin: WandSparkles,
    percival: ShieldCheck,
    loyal_servant: Shield,
    assassin: Swords,
    morgana: Moon,
    mordred: Ghost,
    oberon: EyeOff,
    minion: Skull,
};

/// 角色卡正面：好人走站台主色、壞人紅色
function RoleFace({ role, knownHint, compact = false }: { role: AvalonRole; knownHint: string; compact?: boolean }) {
    const t = useTranslations('Avalon');
    const evil = EVIL_ROLES.has(role);
    const Icon = ROLE_ICON[role];
    return (
        <div className={`flex h-full w-full flex-col items-center justify-center gap-2 rounded-xl border-2 text-center text-white ${compact ? 'p-3' : 'p-5'} ${evil
            ? 'border-red-300/60 bg-linear-to-b from-red-700 to-red-950'
            : 'border-primary-300/60 bg-linear-to-b from-primary-600 to-primary-900'}`}>
            <Icon className={compact ? 'h-8 w-8' : 'h-16 w-16'} strokeWidth={1.5} />
            <p className={`font-bold ${compact ? 'text-lg' : 'text-2xl'}`}>{t(`role_${role}`)}</p>
            <p className="text-xs opacity-80">{evil ? t('sideEvil') : t('sideGood')}</p>
            <p className={`opacity-90 ${compact ? 'text-xs' : 'text-sm'}`}>{knownHint}</p>
        </div>
    );
}

/// 角色卡背面（蓋著）
function RoleBack({ label, compact = false }: { label: string; compact?: boolean }) {
    return (
        <div className={`flex h-full w-full flex-col items-center justify-center gap-2 rounded-xl border-2 border-amber-200/40 bg-[repeating-linear-gradient(45deg,#3b2410_0_8px,#4a2e16_8px_16px)] text-amber-100 ${compact ? 'p-3' : 'p-5'}`}>
            <Fingerprint className={compact ? 'h-8 w-8' : 'h-14 w-14'} strokeWidth={1.5} />
            <p className={compact ? 'text-xs' : 'text-sm'}>{label}</p>
        </div>
    );
}

/// 開局揭示：先蓋著，點一下翻開，看完按「記住了」關掉
export function RoleIntro({ role, knownHint, onDone }: { role: AvalonRole; knownHint: string; onDone: () => void }) {
    const t = useTranslations('Avalon');
    const [open, setOpen] = useState(false);
    return (
        <Modal label={t('youAre')} onClose={onDone} surface="public" backdrop="blur" className="flex flex-col items-center gap-4 p-5">
            <h2 className="text-lg font-bold text-neutral-800 dark:text-neutral-100">{t('introTitle')}</h2>
            <button type="button"
                onClick={() => { if (!open) { sound.warmup(); sound.cardFlip(); setOpen(true); } }}
                className="h-80 w-56" aria-label={open ? t(`role_${role}`) : t('tapToReveal')}>
                {open
                    ? <div key="face" className="piece-flip h-full w-full"><RoleFace role={role} knownHint={knownHint} /></div>
                    : <RoleBack label={t('tapToReveal')} />}
            </button>
            <button type="button" onClick={onDone} disabled={!open}
                className="rounded-lg bg-primary-600 px-6 py-2 font-medium text-white transition-colors hover:bg-primary-700 disabled:opacity-40">
                {t('gotIt')}
            </button>
        </Modal>
    );
}

/// 對局中的小角色卡：平常蓋著，按住才翻開（面對面玩不怕被旁邊的人瞄到）。鍵盤按住 Space / Enter 也可以
export function RolePeek({ role, knownHint }: { role: AvalonRole; knownHint: string }) {
    const t = useTranslations('Avalon');
    const [peek, setPeek] = useState(false);
    const show = () => setPeek(true);
    const hide = () => setPeek(false);
    return (
        <button type="button"
            onPointerDown={show} onPointerUp={hide} onPointerLeave={hide} onPointerCancel={hide}
            onKeyDown={e => { if (e.key === ' ' || e.key === 'Enter') { e.preventDefault(); show(); } }}
            onKeyUp={e => { if (e.key === ' ' || e.key === 'Enter') hide(); }}
            onBlur={hide}
            onContextMenu={e => e.preventDefault()}
            aria-label={t('holdToPeek')}
            className="h-44 w-full touch-none select-none sm:w-36">
            {peek ? <RoleFace role={role} knownHint={knownHint} compact /> : <RoleBack label={t('holdToPeek')} compact />}
        </button>
    );
}
