"use client"

import { useState, useTransition } from 'react'
import { useSearchParams } from 'next/navigation'
import { useRouter, usePathname } from '@/i18n/navigation'
import { useTranslations } from 'next-intl'
import { Search, X } from 'lucide-react'

interface Props {
    q: string
    sort: string
}

// 搜尋（送出時才打）+ 排序（改即打）。皆保留 tag、重設 page。
export default function BlogSearchBar({ q, sort }: Props) {
    const router = useRouter()
    const pathname = usePathname()
    const searchParams = useSearchParams()
    const t = useTranslations('BlogList')
    const [isPending, startTransition] = useTransition()
    const [value, setValue] = useState(q)

    function commit(next: URLSearchParams) {
        next.delete('page')
        startTransition(() => {
            const qs = next.toString()
            router.push(qs ? `${pathname}?${qs}` : pathname)
        })
    }

    function submitSearch(e: React.FormEvent) {
        e.preventDefault()
        const params = new URLSearchParams(searchParams.toString())
        const trimmed = value.trim()
        if (trimmed) params.set('q', trimmed)
        else params.delete('q')
        commit(params)
    }

    function clearSearch() {
        setValue('')
        const params = new URLSearchParams(searchParams.toString())
        params.delete('q')
        commit(params)
    }

    function changeSort(e: React.ChangeEvent<HTMLSelectElement>) {
        const params = new URLSearchParams(searchParams.toString())
        if (e.target.value === 'oldest') params.set('sort', 'oldest')
        else params.delete('sort')
        commit(params)
    }

    return (
        <div className={`flex flex-col sm:flex-row gap-2 items-stretch sm:items-center ${isPending ? 'opacity-50 pointer-events-none' : ''}`}>
            <form onSubmit={submitSearch} className="relative flex-1 min-w-0">
                <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
                <input
                    type="search"
                    value={value}
                    onChange={(e) => setValue(e.target.value)}
                    placeholder={t('searchPlaceholder')}
                    aria-label={t('searchPlaceholder')}
                    className="w-full rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 pl-9 pr-9 py-2 text-sm text-neutral-800 dark:text-neutral-100 placeholder:text-neutral-400 focus:outline-none focus:ring-2 focus:ring-primary-500"
                />
                {value && (
                    <button
                        type="button"
                        onClick={clearSearch}
                        aria-label={t('searchClear')}
                        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded text-neutral-400 hover:text-neutral-700 dark:hover:text-neutral-200"
                    >
                        <X size={16} />
                    </button>
                )}
            </form>
            <select
                value={sort === 'oldest' ? 'oldest' : 'newest'}
                onChange={changeSort}
                aria-label={t('sortLabel')}
                className="rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 px-3 py-2 text-sm text-neutral-700 dark:text-neutral-200 focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
                <option value="newest">{t('sortNewest')}</option>
                <option value="oldest">{t('sortOldest')}</option>
            </select>
        </div>
    )
}
