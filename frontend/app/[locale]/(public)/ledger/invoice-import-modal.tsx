"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { X, Loader2 } from "lucide-react";
import { postInvoice } from "@/api/invoices";
import { parseInvoiceQr, type ParsedInvoice } from "@/libs/invoice-qr";
import InvoiceScanner from "@/components/invoices/invoice-scanner";
import { apiErrorStatus } from "@/libs/api-error";
import useLedgerCategoryLabel from "@/hooks/useLedgerCategoryLabel";
import type { LedgerCategories, InvoiceInput } from "@/types";
import Modal from "@/components/modal";
import { PUBLIC_INPUT } from "@/libs/input-styles";

interface Props {
    categories: LedgerCategories;
    onClose: () => void;
    onImported: () => void;
}


export default function InvoiceImportModal({ categories, onClose, onImported }: Props) {
    const t = useTranslations('Ledger');
    const categoryLabel = useLedgerCategoryLabel();
    const expenseOptions = categories.expense ?? [];
    const defaultCategory = expenseOptions.some(o => o.value === 'other') ? 'other' : (expenseOptions[0]?.value ?? '');

    const [scanKey, setScanKey] = useState(0);
    const [parsed, setParsed] = useState<ParsedInvoice | null>(null);

    // 預覽表單欄位
    const [amount, setAmount] = useState('');
    const [occurredAt, setOccurredAt] = useState('');
    const [category, setCategory] = useState(defaultCategory);
    const [note, setNote] = useState('');
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState('');

    // 回傳 true = 解析成功並接受；false = 非左段 QR，scanner 會靜默續掃
    function handleDecoded(text: string): boolean {
        const result = parseInvoiceQr(text);
        if (!result) return false;
        setParsed(result);
        setAmount(result.amount);
        setOccurredAt(result.occurredAt);
        setNote(result.note ?? '');
        return true;
    }

    function rescan() {
        setParsed(null);
        setError('');
        setScanKey(k => k + 1);
    }

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        if (!parsed) return;
        setSaving(true);
        setError('');
        try {
            // 掃發票記支出 = 登錄發票並帶 record_as_expense（對獎與記帳已解耦走同一前門）
            const input: InvoiceInput = {
                invoice_number: parsed.invoiceNumber,
                invoice_date: occurredAt,
                amount: amount.trim(),
                seller_tax_id: parsed.sellerTaxId,
                source: 'qr',
                record_as_expense: true,
                category,
                note: note.trim() || null,
            };
            await postInvoice(input);
            onImported();
        } catch (err) {
            // 後端訊息是寫死的繁中，印出來等於 en / zh-CN 使用者看到中文；改用 status 分流翻譯
            const status = apiErrorStatus(err);
            if (status === 409) setError(t('alreadyImported'));
            else if (status === 422) setError(t('errorInvalid'));
            else setError(t('errorSave'));
        } finally {
            setSaving(false);
        }
    }

    return (
        <Modal
            label={parsed ? t('importTitle') : t('scanTitle')}
            onClose={onClose}
            size="md"
            surface="public"
            className="max-h-[90vh] overflow-y-auto"
        >
            <div className="flex items-center justify-between px-5 py-4 border-b dark:border-neutral-700">
                <h2 className="font-semibold">{parsed ? t('importTitle') : t('scanTitle')}</h2>
                <button onClick={onClose} className="p-1 rounded-sm hover:bg-neutral-100 dark:hover:bg-neutral-700" aria-label={t('cancel')}>
                    <X size={18} />
                </button>
            </div>

            <div className="p-5">
                {!parsed ? (
                    <div className="flex flex-col gap-3">
                        <InvoiceScanner key={scanKey} mode="qr" onDecoded={handleDecoded} />
                    </div>
                ) : (
                    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                        <div className="text-xs text-neutral-500 dark:text-neutral-400">
                            {t('invoiceNumber')}: <span className="font-mono">{parsed.invoiceNumber}</span>
                            {parsed.sellerTaxId && <> · {t('sellerTaxId')} {parsed.sellerTaxId}</>}
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('amount')}</label>
                                <input type="number" value={amount} onChange={e => setAmount(e.target.value)} required min="0" step="0.01" inputMode="decimal" className={PUBLIC_INPUT} />
                            </div>
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('occurredAt')}</label>
                                <input type="date" value={occurredAt} onChange={e => setOccurredAt(e.target.value)} required className={PUBLIC_INPUT} />
                            </div>
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('category')}</label>
                                <select value={category} onChange={e => setCategory(e.target.value)} required className={PUBLIC_INPUT}>
                                    {expenseOptions.map(o => <option key={o.value} value={o.value}>{categoryLabel(o.value, o.label)}</option>)}
                                </select>
                            </div>
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('note')}</label>
                                <input type="text" value={note} onChange={e => setNote(e.target.value)} maxLength={200} placeholder={t('notePlaceholder')} className={PUBLIC_INPUT} />
                            </div>
                        </div>

                        {error && <p className="text-red-500 text-sm">{error}</p>}

                        <div className="flex gap-2 justify-end">
                            <button type="button" onClick={rescan} className="px-4 py-2 text-sm rounded-sm border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700">
                                {t('rescan')}
                            </button>
                            <button type="submit" disabled={saving} className="flex items-center gap-2 px-4 py-2 text-sm rounded-sm bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50">
                                {saving && <Loader2 className="animate-spin" size={14} />}
                                {t('import')}
                            </button>
                        </div>
                    </form>
                )}
            </div>
        </Modal>
    );
}
