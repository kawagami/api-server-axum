"use client";

import { useLocale } from 'next-intl';
import { useRouter, usePathname } from '@/i18n/navigation';
import { routing } from '@/i18n/routing';

const LOCALE_LABELS: Record<string, string> = {
    'zh-TW': '繁中',
    'zh-CN': '简中',
    'en': 'EN',
};

export default function LocaleSwitcher() {
    const locale = useLocale();
    const router = useRouter();
    const pathname = usePathname();

    const handleChange = (next: string) => {
        router.replace(pathname, { locale: next });
    };

    return (
        <div className="flex items-center gap-1 text-sm">
            {routing.locales.map((l, i) => (
                <span key={l} className="flex items-center gap-1">
                    {i > 0 && <span className="text-neutral-400 dark:text-neutral-600">|</span>}
                    <button
                        onClick={() => handleChange(l)}
                        className={locale === l
                            ? 'font-bold text-primary-500 dark:text-primary-400'
                            : 'hover:text-primary-400 text-neutral-500 dark:text-neutral-400'
                        }
                    >
                        {LOCALE_LABELS[l]}
                    </button>
                </span>
            ))}
        </div>
    );
}
