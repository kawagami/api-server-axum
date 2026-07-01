export default function Loading() {
    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-6">
            <div className="max-w-3xl mx-auto space-y-6">
                <div className="h-8 w-28 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                <div className="flex gap-2">
                    <div className="flex-1 h-10 bg-neutral-200 dark:bg-neutral-700 rounded-md animate-pulse" />
                    <div className="flex-1 h-10 bg-neutral-200 dark:bg-neutral-700 rounded-md animate-pulse" />
                    <div className="h-10 w-20 bg-neutral-200 dark:bg-neutral-700 rounded-md animate-pulse" />
                </div>
                <div className="space-y-2">
                    {Array.from({ length: 5 }).map((_, i) => (
                        <div key={i} className="border rounded-lg dark:border-neutral-700">
                            <div className="flex items-center justify-between px-4 py-3">
                                <div className="flex items-center gap-2 flex-1">
                                    <div className="h-4 w-4 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                                    <div className="h-4 w-32 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                                    <div className="h-4 w-24 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                                </div>
                                <div className="h-6 w-6 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
