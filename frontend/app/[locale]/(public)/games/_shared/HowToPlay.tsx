"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { HelpCircle, X } from 'lucide-react';

// 大廳「怎麼玩」說明彈窗。label/close 走 GameLobby namespace；title + rules 由各遊戲傳入。
export function HowToPlayButton({ title, rules }: { title: string; rules: string[] }) {
    const t = useTranslations('GameLobby');
    const [open, setOpen] = useState(false);

    return (
        <>
            <button
                onClick={() => setOpen(true)}
                className="mx-auto flex items-center gap-1.5 text-sm text-primary-600 transition-colors hover:text-primary-700 hover:underline dark:text-primary-300"
            >
                <HelpCircle className="h-4 w-4" />{t('howToPlay')}
            </button>

            {open && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-neutral-900/70 p-4 backdrop-blur-sm" onClick={() => setOpen(false)}>
                    <div className="flex max-h-[80vh] w-full max-w-md flex-col gap-3 overflow-y-auto rounded-lg bg-white p-5 dark:bg-neutral-800" onClick={e => e.stopPropagation()}>
                        <div className="flex items-center justify-between">
                            <h2 className="text-lg font-bold text-neutral-800 dark:text-neutral-100">{title}</h2>
                            <button onClick={() => setOpen(false)} aria-label={t('close')} className="text-neutral-400 transition-colors hover:text-neutral-600 dark:hover:text-neutral-200">
                                <X className="h-5 w-5" />
                            </button>
                        </div>
                        <ul className="flex list-disc flex-col gap-2 pl-5 text-sm text-neutral-700 dark:text-neutral-200">
                            {rules.map((r, i) => <li key={i}>{r}</li>)}
                        </ul>
                    </div>
                </div>
            )}
        </>
    );
}
