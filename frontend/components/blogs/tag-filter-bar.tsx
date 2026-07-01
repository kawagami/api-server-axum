"use client"

import { useTransition } from 'react'
import { useSearchParams } from 'next/navigation'
import { useRouter, usePathname } from '@/i18n/navigation'
import { useTranslations } from 'next-intl'

interface Props {
    tags: string[]
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

    const btnBase = `text-xs font-semibold px-2.5 py-1 rounded transition-colors text-left ${
        isBar ? 'whitespace-nowrap shrink-0' : ''
    }`

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
                {t('all')}
            </button>
            {tags.map((tag) => (
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
                    {tag}
                </button>
            ))}
        </div>
    )
}
