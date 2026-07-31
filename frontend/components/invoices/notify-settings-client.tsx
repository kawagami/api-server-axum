"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { Loader2, Bell } from "lucide-react";
import { setInvoiceNotify } from "@/api/invoices";

interface Props {
    hasEmail: boolean;
    email: string | null;
    initialEnabled: boolean;
}

export default function NotifySettingsClient({ hasEmail, email, initialEnabled }: Props) {
    const t = useTranslations('Invoices');
    // 初始值讀 /members/me 的 lottery_notify_enabled；PATCH 回傳值作為互動後真值
    const [enabled, setEnabled] = useState(initialEnabled);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState('');

    async function toggle() {
        if (!hasEmail || saving) return;
        const next = !enabled;
        setSaving(true);
        setError('');
        try {
            const res = await setInvoiceNotify(next);
            setEnabled(res.enabled);
        } catch (err) {
            const e2 = err as Error & { status?: number; errorData?: { message?: string } };
            if (e2.status === 422) setError(t('notifyNeedsEmail'));
            else setError(e2.errorData?.message || t('errorSave'));
        } finally {
            setSaving(false);
        }
    }

    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700 flex flex-col gap-4">
            <div className="flex items-start gap-3">
                <Bell className="text-primary-500 shrink-0 mt-0.5" size={20} />
                <div className="flex flex-col gap-1 min-w-0 flex-1">
                    <span className="font-medium">{t('notifyTitle')}</span>
                    <span className="text-sm text-neutral-500 dark:text-neutral-400">{t('notifyDesc')}</span>
                    {hasEmail && email && (
                        <span className="text-xs text-neutral-400 dark:text-neutral-500">{t('notifyTarget', { email })}</span>
                    )}
                </div>
                <button
                    onClick={toggle}
                    disabled={!hasEmail || saving}
                    role="switch"
                    aria-checked={enabled}
                    aria-label={t('notifyTitle')}
                    className={`relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${enabled ? 'bg-primary-500' : 'bg-neutral-300 dark:bg-neutral-600'}`}
                >
                    {saving
                        ? <Loader2 className="animate-spin mx-auto text-white" size={14} />
                        : <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${enabled ? 'translate-x-6' : 'translate-x-1'}`} />}
                </button>
            </div>

            {!hasEmail && (
                <p className="text-sm text-neutral-500 dark:text-neutral-400 border-t border-neutral-100 dark:border-neutral-700 pt-3">
                    {t('notifyNoEmail')}
                </p>
            )}
            {error && <p className="text-red-500 text-sm">{error}</p>}
            <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('disclaimer')}</p>
        </div>
    );
}
