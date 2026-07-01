"use client";

import { useEffect, useRef, useState } from 'react';
import { useTranslations } from 'next-intl';
import { Send } from 'lucide-react';
import type { ChatEntry } from './avalon-types';

export function AvalonChat({ chat, onSend }: { chat: ChatEntry[]; onSend: (text: string) => void }) {
    const t = useTranslations('Avalon');
    const [text, setText] = useState('');
    const endRef = useRef<HTMLDivElement>(null);

    useEffect(() => { endRef.current?.scrollIntoView({ block: 'end' }); }, [chat]);

    const submit = (e: React.FormEvent) => {
        e.preventDefault();
        const v = text.trim();
        if (!v) return;
        onSend(v.slice(0, 500));
        setText('');
    };

    return (
        <div className="flex min-h-0 flex-1 flex-col rounded-lg border border-neutral-200 dark:border-neutral-700">
            <div className="min-h-0 flex-1 overflow-y-auto p-2 text-sm">
                {chat.length === 0 ? (
                    <p className="text-neutral-400">{t('chatEmpty')}</p>
                ) : chat.map(c => (
                    <p key={c.id} className="break-words">
                        <span className="font-semibold text-primary-700 dark:text-primary-300">{c.name}</span>
                        <span className="text-neutral-700 dark:text-neutral-200">：{c.text}</span>
                    </p>
                ))}
                <div ref={endRef} />
            </div>
            <form onSubmit={submit} className="flex gap-1 border-t border-neutral-200 p-2 dark:border-neutral-700">
                <input
                    value={text}
                    onChange={e => setText(e.target.value)}
                    maxLength={500}
                    placeholder={t('chatPlaceholder')}
                    className="min-w-0 flex-1 rounded-md border border-neutral-300 bg-white px-2 py-1 text-sm text-neutral-800 focus:border-primary-400 focus:outline-none dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-100"
                />
                <button type="submit" className="flex flex-none items-center rounded-md bg-primary-600 px-3 text-white transition-colors hover:bg-primary-700">
                    <Send className="h-4 w-4" />
                </button>
            </form>
        </div>
    );
}
