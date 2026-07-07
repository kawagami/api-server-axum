"use client"

import { useTransition } from 'react'
import { useSearchParams } from 'next/navigation'
import { useRouter, usePathname } from '@/i18n/navigation'
import { useTranslations } from 'next-intl'
import type { TagCount } from '@/types'

interface Props {
    tags: TagCount[]
    selectedTag: string | null
    // sidebar：桌機右側直向；bar：手機上方橫向可捲動
    variant?: 'sidebar' | 'bar'
}

export default function TagFilterBar({ tags, selectedTag, variant = 'sidebar' }: Props) {
    const router = useRouter()
    const pathname = usePathname()
    const searchParams = useSearchParams()
    const t = useTranslations('BlogList')
    const [isPending, startTransition] = useTransition()

    function navigate(params: URLSearchParams) {
        startTransition(() => {
            router.push(`${pathname}?${params.toString()}`)
        })
    }

    function selectTag(tag: string) {
        const params = new URLSearchParams(searchParams.toString())
        params.delete('page')
        if (selectedTag === tag) {
            params.delete('tag')
        } else {
            params.set('tag', tag)
        }
        navigate(params)
    }

    function clearTag() {
        const params = new URLSearchParams(searchParams.toString())
        params.delete('page')
        params.delete('tag')
        navigate(params)
    }

    const isBar = variant === 'bar'
    const container = isBar
        ? 'flex gap-1.5 overflow-x-auto pb-1'
        : 'sticky top-2 flex flex-col gap-1.5'

    const btnBase = `text-xs font-semibold px-2.5 py-1 rounded transition-colors ${
        isBar ? 'whitespace-nowrap shrink-0' : 'flex items-center justify-between gap-2 text-left'
    }`

    const totalCount = tags.reduce((sum, { count }) => sum + count, 0)

    return (
        <div className={`${container} ${isPending ? 'opacity-50 pointer-events-none' : ''}`}>
            <button
                onClick={clearTag}
                aria-pressed={selectedTag === null}
                className={`${btnBase} ${
                    selectedTag === null
                        ? 'bg-primary-600 text-white dark:bg-primary-500'
                        : 'bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300 hover:bg-primary-200 dark:hover:bg-primary-800'
                }`}
            >
                <span>{t('all')}</span>
                {!isBar && <span className="opacity-70 tabular-nums">{totalCount}</span>}
            </button>
            {tags.map(({ tag, count }) => (
                <button
                    key={tag}
                    onClick={() => selectTag(tag)}
                    aria-pressed={selectedTag === tag}
                    className={`${btnBase} ${
                        selectedTag === tag
                            ? 'bg-primary-600 text-white dark:bg-primary-500'
                            : 'bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300 hover:bg-primary-200 dark:hover:bg-primary-800'
                    }`}
                >
                    <span className={isBar ? '' : 'truncate'}>{tag}</span>
                    {!isBar && <span className="opacity-70 tabular-nums shrink-0">{count}</span>}
                </button>
            ))}
        </div>
    )
}
