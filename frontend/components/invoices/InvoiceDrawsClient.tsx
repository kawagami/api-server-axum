"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";
import type { InvoiceDraw } from "@/types";

// period = 'YYYYMM'（期末偶數月）→ 'YYYY / (M-1)–M'
function formatPeriod(period: string): string {
    if (period.length !== 6) return period;
    const year = period.slice(0, 4);
    const end = parseInt(period.slice(4), 10);
    const start = end - 1;
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${year} / ${pad(start)}–${pad(end)}`;
}

// 8 碼中獎號碼：末 3 碼加粗（六獎起即比末 3 碼，愈多碼相符獎愈高）
function InvoiceNumber({ value }: { value: string }) {
    const head = value.slice(0, -3);
    const tail = value.slice(-3);
    return (
        <span className="font-mono tabular-nums tracking-wide">
            <span className="text-neutral-500 dark:text-neutral-400">{head}</span>
            <span className="font-bold text-primary-600 dark:text-primary-400">{tail}</span>
        </span>
    );
}

function Row({ label, children }: { label: string; children: ReactNode }) {
    return (
        <div className="flex items-baseline gap-2">
            <dt className="w-16 shrink-0 text-neutral-500 dark:text-neutral-400">{label}</dt>
            <dd className="flex flex-col gap-0.5">{children}</dd>
        </div>
    );
}

export default function InvoiceDrawsClient({ initialDraws }: { initialDraws: InvoiceDraw[] }) {
    const t = useTranslations('Invoices');

    if (initialDraws.length === 0) {
        return <p className="text-center text-neutral-500 dark:text-neutral-400 py-12">{t('noDraws')}</p>;
    }

    return (
        <div className="flex flex-col gap-4">
            <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('drawsHint')}</p>

            {initialDraws.map(d => (
                <div
                    key={d.period}
                    className="bg-white dark:bg-neutral-800 rounded-xl px-4 py-3 shadow border dark:border-neutral-700 flex flex-col gap-2"
                >
                    <div className="flex items-center gap-2 text-sm">
                        <span className="font-medium">{t('period')}</span>
                        <span className="font-mono text-neutral-500 dark:text-neutral-400">{formatPeriod(d.period)}</span>
                    </div>
                    <dl className="flex flex-col gap-1.5 text-sm">
                        {d.special && (
                            <Row label={t('prizeSpecial')}>
                                <InvoiceNumber value={d.special} />
                            </Row>
                        )}
                        {d.grand && (
                            <Row label={t('prizeGrand')}>
                                <InvoiceNumber value={d.grand} />
                            </Row>
                        )}
                        {d.first.length > 0 && (
                            <Row label={t('prizeFirst')}>
                                {d.first.map((n, i) => <InvoiceNumber key={i} value={n} />)}
                            </Row>
                        )}
                        {d.additional.length > 0 && (
                            <Row label={t('prizeAdditionalSixth')}>
                                <span className="flex flex-wrap gap-2 font-mono font-bold text-primary-600 dark:text-primary-400">
                                    {d.additional.map((n, i) => <span key={i}>{n}</span>)}
                                </span>
                            </Row>
                        )}
                    </dl>
                </div>
            ))}

            <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('disclaimer')}</p>
        </div>
    );
}
