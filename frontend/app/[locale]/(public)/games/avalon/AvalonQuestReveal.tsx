"use client";

import { useEffect, useEffectEvent, useState } from 'react';
import { useTranslations } from 'next-intl';
import { Check, X } from 'lucide-react';
import Modal from '@/components/modal';
import { sound } from '../_shared/sound';

const FIRST_MS = 500;   // 第一張翻開前的停頓
const STEP_MS = 650;    // 每張間隔
const RESULT_MS = 350;  // 最後一張翻完到顯示結果
const CLOSE_MS = 2200;  // 顯示結果後自動關閉

/// 失敗卡放在哪幾張：用輪次做確定性的打散。server 只給失敗張數（本來就匿名），
/// 這裡的順序純屬演出，每個人看到的一樣也無妨；不用 Math.random 是為了 render 保持純函式
function cardOrder(size: number, fails: number, round: number): boolean[] {
    const idx = Array.from({ length: size }, (_, i) => i)
        .sort((a, b) => ((a + 1) * 7919 + round * 104729) % 97 - ((b + 1) * 7919 + round * 104729) % 97);
    const failAt = new Set(idx.slice(0, fails));
    return Array.from({ length: size }, (_, i) => !failAt.has(i)); // true = 成功卡
}

/// 任務卡逐張翻開，最後揭曉成功／失敗。點背景或 Esc 可跳過；減少動態效果時直接全開
export function AvalonQuestReveal({ round, size, fails, success, onDone }: {
    round: number; size: number; fails: number; success: boolean; onDone: () => void;
}) {
    const t = useTranslations('Avalon');
    const [reduced] = useState(() => window.matchMedia('(prefers-reduced-motion: reduce)').matches);
    const [shown, setShown] = useState(reduced ? size : 0);
    const [finished, setFinished] = useState(reduced);
    const cards = cardOrder(size, fails, round);

    const flip = useEffectEvent((i: number) => {
        setShown(i + 1);
        if (cards[i]) sound.cardFlip(); else sound.failCard();
    });
    const outcomeSound = useEffectEvent(() => {
        if (success) sound.questSuccess(); else sound.questFail();
    });
    const finish = useEffectEvent(() => {
        setFinished(true);
        outcomeSound();
    });
    const close = useEffectEvent(() => onDone());

    useEffect(() => {
        if (reduced) {
            outcomeSound(); // 減少動態效果：結果一開始就顯示（finished 初值為 true），只補音效
            const timer = setTimeout(close, CLOSE_MS);
            return () => clearTimeout(timer);
        }
        const timers = Array.from({ length: size }, (_, i) => setTimeout(() => flip(i), FIRST_MS + i * STEP_MS));
        const end = FIRST_MS + size * STEP_MS + RESULT_MS;
        timers.push(setTimeout(finish, end));
        timers.push(setTimeout(close, end + CLOSE_MS));
        return () => timers.forEach(clearTimeout);
    }, [reduced, size]);

    return (
        <Modal label={t('questReveal', { n: round + 1 })} onClose={onDone} surface="public" backdrop="blur" size="lg"
            className="flex flex-col items-center gap-5 p-6">
            <h2 className="text-lg font-bold text-neutral-800 dark:text-neutral-100">{t('questReveal', { n: round + 1 })}</h2>
            <div className="flex flex-wrap justify-center gap-2 sm:gap-3">
                {cards.map((ok, i) => i < shown ? (
                    <div key={`f${i}`} className={`piece-flip flex h-24 w-16 items-center justify-center rounded-lg border-2 text-white shadow-lg sm:h-28 sm:w-20 ${ok
                        ? 'border-primary-200 bg-linear-to-b from-primary-500 to-primary-800'
                        : 'border-red-300 bg-linear-to-b from-red-600 to-red-900'}`}>
                        {ok ? <Check className="h-9 w-9" strokeWidth={3} /> : <X className="h-9 w-9" strokeWidth={3} />}
                    </div>
                ) : (
                    <div key={`b${i}`} className="h-24 w-16 rounded-lg border-2 border-amber-200/40 bg-[repeating-linear-gradient(45deg,#3b2410_0_8px,#4a2e16_8px_16px)] shadow-lg sm:h-28 sm:w-20" />
                ))}
            </div>
            <p aria-live="polite" className={`min-h-8 text-2xl font-bold transition-opacity ${finished ? 'opacity-100' : 'opacity-0'} ${success ? 'text-primary-600 dark:text-primary-300' : 'text-red-600 dark:text-red-400'}`}>
                {finished && (success ? t('questSuccess') : t('questFail'))}
            </p>
        </Modal>
    );
}
