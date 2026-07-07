// 公開文章列表載入骨架，對齊 blog-list.tsx 的版面（搜尋列 + 卡片 + 側欄）
export default function BlogsLoading() {
    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-hidden">
            <div className="max-w-4xl mx-auto px-4">
                {/* 搜尋 + 排序 */}
                <div className="pt-4 flex flex-col sm:flex-row gap-2">
                    <div className="h-10 flex-1 rounded-lg bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                    <div className="h-10 w-full sm:w-28 rounded-lg bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                </div>
                <div className="flex gap-6 pt-3">
                    <div className="flex-1 min-w-0">
                        {Array.from({ length: 4 }).map((_, i) => (
                            <div
                                key={i}
                                className="bg-white dark:bg-neutral-800 shadow-md rounded-xl m-4 p-5 space-y-3"
                            >
                                <div className="h-5 w-2/3 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                                <div className="h-4 w-full rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                                <div className="h-4 w-4/5 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                                <div className="flex gap-2 pt-1">
                                    <div className="h-4 w-12 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                                    <div className="h-4 w-16 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                                </div>
                            </div>
                        ))}
                    </div>
                    <aside className="w-44 shrink-0 pt-1 hidden sm:block space-y-1.5">
                        {Array.from({ length: 6 }).map((_, i) => (
                            <div
                                key={i}
                                className="h-7 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse"
                            />
                        ))}
                    </aside>
                </div>
            </div>
        </div>
    );
}
