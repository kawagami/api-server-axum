"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import type { LedgerEntry, LedgerInput, LedgerKind, LedgerCategories } from "@/types";

interface Props {
    categories: LedgerCategories;
    initial?: LedgerEntry;
    onSave: (input: LedgerInput) => Promise<void>;
    onCancel: () => void;
}

// 台北時區今日 YYYY-MM-DD（新增表單預設值）
function todayTaipei() {
    return new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Taipei' }).format(new Date());
}

const inputClass = "border rounded px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400";

export default function LedgerForm({ categories, initial, onSave, onCancel }: Props) {
    const t = useTranslations('Ledger');
    const [kind, setKind] = useState<LedgerKind>(initial?.kind ?? 'expense');
    const [amount, setAmount] = useState(initial?.amount ?? '');
    const [category, setCategory] = useState(initial?.category ?? '');
    const [note, setNote] = useState(initial?.note ?? '');
    const [occurredAt, setOccurredAt] = useState(initial?.occurred_at ?? todayTaipei());
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState('');

    const options = categories[kind] ?? [];

    function handleKindChange(next: LedgerKind) {
        setKind(next);
        // 切換 kind 後若原分類不在新清單，清空
        if (!(categories[next] ?? []).some(o => o.value === category)) {
            setCategory('');
        }
    }

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        if (Number(amount) <= 0) {
            setError(t('errorAmount'));
            return;
        }
        setSaving(true);
        setError('');
        try {
            await onSave({
                kind,
                amount: amount.trim(),
                category,
                note: note.trim() || null,
                occurred_at: occurredAt,
            });
        } catch (e) {
            const msg = (e as { errorData?: { message?: string } }).errorData?.message;
            setError(msg || t('errorSave'));
        } finally {
            setSaving(false);
        }
    }

    return (
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            {/* kind 切換 */}
            <div className="flex gap-2">
                {(['expense', 'income'] as const).map(k => (
                    <button
                        key={k}
                        type="button"
                        onClick={() => handleKindChange(k)}
                        className={`flex-1 py-2 text-sm rounded border transition-colors ${kind === k
                            ? k === 'income'
                                ? 'border-green-500 bg-green-500/10 text-green-600 dark:text-green-400 font-medium'
                                : 'border-red-500 bg-red-500/10 text-red-600 dark:text-red-400 font-medium'
                            : 'border-neutral-300 dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700'
                            }`}
                    >
                        {t(k)}
                    </button>
                ))}
            </div>

            <div className="grid grid-cols-2 gap-4">
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('amount')}</label>
                    <input
                        type="number"
                        value={amount}
                        onChange={e => setAmount(e.target.value)}
                        required
                        min="0"
                        step="0.01"
                        inputMode="decimal"
                        placeholder="0.00"
                        className={inputClass}
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('category')}</label>
                    <select
                        value={category}
                        onChange={e => setCategory(e.target.value)}
                        required
                        className={inputClass}
                    >
                        <option value="" disabled>{t('selectCategory')}</option>
                        {options.map(o => (
                            <option key={o.value} value={o.value}>{o.label}</option>
                        ))}
                    </select>
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('occurredAt')}</label>
                    <input
                        type="date"
                        value={occurredAt}
                        onChange={e => setOccurredAt(e.target.value)}
                        required
                        className={inputClass}
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('note')}</label>
                    <input
                        type="text"
                        value={note}
                        onChange={e => setNote(e.target.value)}
                        maxLength={200}
                        placeholder={t('notePlaceholder')}
                        className={inputClass}
                    />
                </div>
            </div>

            {error && <p className="text-red-500 text-sm">{error}</p>}

            <div className="flex gap-2 justify-end">
                <button
                    type="button"
                    onClick={onCancel}
                    className="px-4 py-2 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                >
                    {t('cancel')}
                </button>
                <button
                    type="submit"
                    disabled={saving}
                    className="px-4 py-2 text-sm rounded bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50"
                >
                    {saving ? '...' : t('save')}
                </button>
            </div>
        </form>
    );
}
