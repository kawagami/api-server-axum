"use client"

import { useTransition } from 'react'
import { useSearchParams } from 'next/navigation'
import { useRouter, usePathname } from '@/i18n/navigation'
import { useTranslations } from 'next-intl'
import { ChevronLeft, ChevronRight } from 'lucide-react'

interface Props {
    page: number
    totalPages: number
}

// 產生頁碼序列，過多時以 '…' 省略：永遠含第一頁/末頁與當前頁前後各一
function pageItems(page: number, totalPages: number): (number | 'gap')[] {
    if (totalPages <= 7) {
        return Array.from({ length: totalPages }, (_, i) => i + 1)
    }
    const items: (number | 'gap')[] = [1]
    const start = Math.max(2, page - 1)
    const end = Math.min(totalPages - 1, page + 1)
    if (start > 2) items.push('gap')
    for (let p = start; p <= end; p++) items.push(p)
    if (end < totalPages - 1) items.push('gap')
    items.push(totalPages)
    return items
}

export default function Pagination({ page, totalPages }: Props) {
    const router = useRouter()
    const pathname = usePathname()
    const searchParams = useSearchParams()
    const t = useTranslations('Pagination')
    const [isPending, startTransition] = useTransition()

    if (totalPages <= 1) return null

    function goTo(p: number) {
        const params = new URLSearchParams(searchParams.toString())
        params.set('page', String(p))
        startTransition(() => {
            router.push(`${pathname}?${params.toString()}`)
        })
    }

    const arrowBtn =
        'p-1 rounded text-neutral-500 dark:text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed'

    return (
        <div className={`flex items-center justify-center gap-1.5 py-4 flex-wrap ${isPending ? 'opacity-50' : ''}`}>
            <button
                onClick={() => goTo(page - 1)}
                disabled={page <= 1 || isPending}
                aria-label={t('prev')}
                className={arrowBtn}
            >
                <ChevronLeft size={20} />
            </button>
            {pageItems(page, totalPages).map((item, i) =>
                item === 'gap' ? (
                    <span key={`gap-${i}`} className="px-1 text-sm text-neutral-400 select-none">…</span>
                ) : (
                    <button
                        key={item}
                        onClick={() => goTo(item)}
                        disabled={isPending}
                        aria-current={item === page ? 'page' : undefined}
                        className={`min-w-8 h-8 px-2 rounded text-sm transition-colors ${
                            item === page
                                ? 'bg-primary-600 text-white dark:bg-primary-500 font-semibold'
                                : 'text-neutral-600 dark:text-neutral-400 hover:bg-primary-100 dark:hover:bg-primary-900'
                        }`}
                    >
                        {item}
                    </button>
                )
            )}
            <button
                onClick={() => goTo(page + 1)}
                disabled={page >= totalPages || isPending}
                aria-label={t('next')}
                className={arrowBtn}
            >
                <ChevronRight size={20} />
            </button>
        </div>
    )
}
