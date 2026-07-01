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

    return (
        <div className={`flex items-center justify-center gap-3 py-4 ${isPending ? 'opacity-50' : ''}`}>
            <button
                onClick={() => goTo(page - 1)}
                disabled={page <= 1 || isPending}
                aria-label={t('prev')}
                className="p-1 rounded text-neutral-500 dark:text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed"
            >
                <ChevronLeft size={20} />
            </button>
            <span className="text-sm text-neutral-600 dark:text-neutral-400">
                {page} / {totalPages}
            </span>
            <button
                onClick={() => goTo(page + 1)}
                disabled={page >= totalPages || isPending}
                aria-label={t('next')}
                className="p-1 rounded text-neutral-500 dark:text-neutral-400 hover:text-neutral-800 dark:hover:text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed"
            >
                <ChevronRight size={20} />
            </button>
        </div>
    )
}
