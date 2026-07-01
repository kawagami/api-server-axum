"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { Link } from "@/i18n/navigation";
import { QrCode, ScanBarcode, Keyboard, Loader2, CheckCircle2 } from "lucide-react";
import { postInvoice } from "@/api/invoices";
import { parseInvoiceQr } from "@/libs/invoice-qr";
import { parseInvoiceBarcode } from "@/libs/invoice-barcode";
import InvoiceScanner from "@/components/invoices/InvoiceScanner";
import type { LedgerCategories, InvoiceInput, InvoiceSource } from "@/types";

interface Props {
    categories: LedgerCategories;
}

type Tab = 'qr' | 'barcode' | 'manual';

const inputClass = "border rounded px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400";
const INVOICE_RE = /^[A-Z]{2}\d{8}$/;

export default function InvoiceRegisterClient({ categories }: Props) {
    const t = useTranslations('Invoices');
    const expenseOptions = categories.expense ?? [];
    const defaultCategory = expenseOptions.some(o => o.value === 'other') ? 'other' : (expenseOptions[0]?.value ?? '');

    const [tab, setTab] = useState<Tab>('qr');
    const [scanKey, setScanKey] = useState(0);
    // 是否已有資料可填表（掃描成功 / 手動）
    const [ready, setReady] = useState(false);

    // 表單欄位
    const [invoiceNumber, setInvoiceNumber] = useState('');
    const [invoiceDate, setInvoiceDate] = useState('');
    const [amount, setAmount] = useState('');
    const [sellerTaxId, setSellerTaxId] = useState<string | null>(null);
    const [recordAsExpense, setRecordAsExpense] = useState(false);
    const [category, setCategory] = useState(defaultCategory);
    const [note, setNote] = useState('');

    const [saving, setSaving] = useState(false);
    const [error, setError] = useState('');
    const [done, setDone] = useState(false);

    function resetForm() {
        setInvoiceNumber('');
        setInvoiceDate('');
        setAmount('');
        setSellerTaxId(null);
        setRecordAsExpense(false);
        setCategory(defaultCategory);
        setNote('');
        setError('');
        setReady(false);
    }

    function switchTab(next: Tab) {
        setTab(next);
        setDone(false);
        resetForm();
        if (next === 'manual') setReady(true);
        else setScanKey(k => k + 1);
    }

    // 回傳 true = 解析成功並接受；false = 非左段/非發票碼，scanner 會靜默續掃
    function handleDecoded(text: string): boolean {
        if (tab === 'qr') {
            const r = parseInvoiceQr(text);
            if (!r) return false;
            setInvoiceNumber(r.invoiceNumber);
            setInvoiceDate(r.occurredAt);
            setAmount(r.amount);
            setSellerTaxId(r.sellerTaxId);
            setNote(r.note ?? '');
        } else {
            const r = parseInvoiceBarcode(text);
            if (!r) return false;
            setInvoiceNumber(r.invoiceNumber);
            setInvoiceDate(r.occurredAt);
        }
        setReady(true);
        return true;
    }

    function rescan() {
        resetForm();
        setScanKey(k => k + 1);
    }

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        const number = invoiceNumber.trim().toUpperCase();
        if (!INVOICE_RE.test(number)) {
            setError(t('invalidNumber'));
            return;
        }
        if (!invoiceDate) {
            setError(t('invalidDate'));
            return;
        }
        if (recordAsExpense && !amount.trim()) {
            setError(t('expenseNeedsAmount'));
            return;
        }
        setSaving(true);
        setError('');
        try {
            const source: InvoiceSource = tab;
            const input: InvoiceInput = {
                invoice_number: number,
                invoice_date: invoiceDate,
                amount: amount.trim() || null,
                seller_tax_id: sellerTaxId,
                source,
                record_as_expense: recordAsExpense,
                ...(recordAsExpense ? { category, note: note.trim() || null } : {}),
            };
            await postInvoice(input);
            setDone(true);
        } catch (err) {
            const e2 = err as Error & { status?: number; errorData?: { message?: string } };
            if (e2.status === 409) setError(t('alreadyRegistered'));
            else setError(e2.errorData?.message || t('errorSave'));
        } finally {
            setSaving(false);
        }
    }

    if (done) {
        return (
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-8 shadow border dark:border-neutral-700 flex flex-col items-center gap-4 text-center">
                <CheckCircle2 className="text-green-500" size={48} />
                <p className="font-semibold">{t('registered')}</p>
                <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('disclaimer')}</p>
                <div className="flex gap-2">
                    <button
                        onClick={() => switchTab(tab)}
                        className="px-4 py-2 text-sm rounded bg-primary-500 text-white hover:bg-primary-600"
                    >
                        {t('registerAnother')}
                    </button>
                    <Link
                        href="/invoices"
                        className="px-4 py-2 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                    >
                        {t('viewMyInvoices')}
                    </Link>
                </div>
            </div>
        );
    }

    const tabs: { key: Tab; labelKey: string; icon: typeof QrCode }[] = [
        { key: 'qr', labelKey: 'tabQr', icon: QrCode },
        { key: 'barcode', labelKey: 'tabBarcode', icon: ScanBarcode },
        { key: 'manual', labelKey: 'tabManual', icon: Keyboard },
    ];

    return (
        <div className="flex flex-col gap-4">
            {/* 三入口 tab */}
            <div className="grid grid-cols-3 gap-1.5 sm:gap-2">
                {tabs.map(({ key, labelKey, icon: Icon }) => (
                    <button
                        key={key}
                        onClick={() => switchTab(key)}
                        className={`flex flex-col items-center gap-1 py-3 rounded-xl border text-xs sm:text-sm transition-colors ${tab === key
                            ? 'border-primary-500 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-300'
                            : 'border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}`}
                    >
                        <Icon size={20} />
                        {t(labelKey)}
                    </button>
                ))}
            </div>

            <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700">
                {!ready ? (
                    <div className="flex flex-col gap-3">
                        <InvoiceScanner key={`${tab}-${scanKey}`} mode={tab === 'barcode' ? 'barcode' : 'qr'} onDecoded={handleDecoded} />
                    </div>
                ) : (
                    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('invoiceNumber')}</label>
                                <input
                                    type="text"
                                    value={invoiceNumber}
                                    onChange={e => setInvoiceNumber(e.target.value.toUpperCase())}
                                    required
                                    maxLength={10}
                                    placeholder="AB12345678"
                                    className={`${inputClass} font-mono`}
                                />
                            </div>
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('invoiceDate')}</label>
                                <input type="date" value={invoiceDate} onChange={e => setInvoiceDate(e.target.value)} required className={inputClass} />
                            </div>
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('amount')} <span className="text-neutral-400 font-normal">({t('optional')})</span></label>
                                <input type="number" value={amount} onChange={e => setAmount(e.target.value)} min="0" step="0.01" inputMode="decimal" className={inputClass} />
                            </div>
                            <div className="flex flex-col gap-1">
                                <label className="text-sm font-medium">{t('sellerTaxId')} <span className="text-neutral-400 font-normal">({t('optional')})</span></label>
                                <input
                                    type="text"
                                    value={sellerTaxId ?? ''}
                                    onChange={e => setSellerTaxId(e.target.value.trim() || null)}
                                    maxLength={8}
                                    inputMode="numeric"
                                    className={`${inputClass} font-mono`}
                                />
                            </div>
                        </div>

                        {/* 同時記為支出 */}
                        <label className="flex items-center gap-2 text-sm cursor-pointer">
                            <input type="checkbox" checked={recordAsExpense} onChange={e => setRecordAsExpense(e.target.checked)} className="accent-primary-500" />
                            {t('recordAsExpense')}
                        </label>
                        {recordAsExpense && (
                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <div className="flex flex-col gap-1">
                                    <label className="text-sm font-medium">{t('category')}</label>
                                    <select value={category} onChange={e => setCategory(e.target.value)} className={inputClass}>
                                        {expenseOptions.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
                                    </select>
                                </div>
                                <div className="flex flex-col gap-1">
                                    <label className="text-sm font-medium">{t('note')}</label>
                                    <input type="text" value={note} onChange={e => setNote(e.target.value)} maxLength={200} placeholder={t('optional')} className={inputClass} />
                                </div>
                            </div>
                        )}

                        {error && <p className="text-red-500 text-sm">{error}</p>}
                        <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('disclaimer')}</p>

                        <div className="flex gap-2 justify-end">
                            {tab !== 'manual' && (
                                <button type="button" onClick={rescan} className="px-4 py-2 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700">
                                    {t('rescan')}
                                </button>
                            )}
                            <button type="submit" disabled={saving} className="flex items-center gap-2 px-4 py-2 text-sm rounded bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50">
                                {saving && <Loader2 className="animate-spin" size={14} />}
                                {t('register')}
                            </button>
                        </div>
                    </form>
                )}
            </div>
        </div>
    );
}
