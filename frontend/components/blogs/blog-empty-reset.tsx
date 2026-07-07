"use client"

import { useTransition } from 'react'
import { useRouter, usePathname } from '@/i18n/navigation'
import { useTranslations } from 'next-intl'

// 空狀態 CTA：清除 tag / 關鍵字 / 分頁（保留 author，因它在路徑而非 query）
export default function BlogEmptyReset() {
    const router = useRouter()
    const pathname = usePathname()
    const t = useTranslations('BlogList')
    const [isPending, startTransition] = useTransition()

    return (
        <button
            onClick={() => startTransition(() => router.push(pathname))}
            disabled={isPending}
            className="mt-3 text-sm font-semibold text-primary-600 dark:text-primary-300 hover:underline disabled:opacity-50"
        >
            {t('clearFilters')}
        </button>
    )
}
