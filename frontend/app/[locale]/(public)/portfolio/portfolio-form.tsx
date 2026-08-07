"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import type { PortfolioEntry, PortfolioEntryInput } from "@/types";

interface Props {
    initial?: PortfolioEntry;
    onSave: (input: PortfolioEntryInput) => Promise<void>;
    onCancel: () => void;
}

// 權威在後端（structs/portfolio.rs 的 MIN_BUY_YEAR + PortfolioRequest::validate，
// 超出範圍回 422）。這裡只是即時回饋：buy_date 是後端逐月抓價迴圈的起點，
// 打錯一個世紀會變成上萬次查詢，讓使用者當場看到比吞成通用「儲存失敗」好。
const MIN_BUY_DATE = '1992-01-01';

// 台北日，與後端的 taipei_today() 對齊；用瀏覽器當地日會讓 UTC+9 以東的使用者
// 選到後端眼中的「明天」而被 422。en-CA 的輸出剛好是 YYYY-MM-DD。
function taipeiToday(): string {
    return new Intl.DateTimeFormat('en-CA', { timeZone: 'Asia/Taipei' }).format(new Date());
}

export default function PortfolioForm({ initial, onSave, onCancel }: Props) {
    const t = useTranslations('Portfolio');
    const [stockCode, setStockCode] = useState(initial?.stock_code ?? '');
    const [buyDate, setBuyDate] = useState(initial?.buy_date ?? '');
    const [costPerShare, setCostPerShare] = useState(initial?.cost_per_share?.toString() ?? '');
    const [shares, setShares] = useState(initial?.shares?.toString() ?? '');
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState('');

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        setSaving(true);
        setError('');
        try {
            await onSave({
                stock_code: stockCode.trim().toUpperCase(),
                buy_date: buyDate,
                cost_per_share: parseFloat(costPerShare),
                shares: parseInt(shares, 10),
            });
        } catch {
            setError(t('errorSave'));
        } finally {
            setSaving(false);
        }
    }

    return (
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <div className="grid grid-cols-2 gap-4">
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('stockCode')}</label>
                    <input
                        type="text"
                        value={stockCode}
                        onChange={e => setStockCode(e.target.value)}
                        required
                        placeholder="e.g. 2330"
                        className="border rounded-sm px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600"
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('buyDate')}</label>
                    <input
                        type="date"
                        value={buyDate}
                        onChange={e => setBuyDate(e.target.value)}
                        required
                        min={MIN_BUY_DATE}
                        max={taipeiToday()}
                        className="border rounded-sm px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600"
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('costPerShare')}</label>
                    <input
                        type="number"
                        value={costPerShare}
                        onChange={e => setCostPerShare(e.target.value)}
                        required
                        min="0"
                        step="0.01"
                        className="border rounded-sm px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600"
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm font-medium">{t('shares')}</label>
                    <input
                        type="number"
                        value={shares}
                        onChange={e => setShares(e.target.value)}
                        required
                        min="1"
                        step="1"
                        className="border rounded-sm px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600"
                    />
                </div>
            </div>

            {error && <p className="text-red-500 text-sm">{error}</p>}

            <div className="flex gap-2 justify-end">
                <button
                    type="button"
                    onClick={onCancel}
                    className="px-4 py-2 text-sm rounded-sm border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                >
                    {t('cancel')}
                </button>
                <button
                    type="submit"
                    disabled={saving}
                    className="px-4 py-2 text-sm rounded-sm bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50"
                >
                    {saving ? '...' : t('save')}
                </button>
            </div>
        </form>
    );
}
