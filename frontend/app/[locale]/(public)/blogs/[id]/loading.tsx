export default function BlogLoading() {
    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto px-4 py-6">
            <div className="max-w-prose mx-auto animate-pulse space-y-4">
                <div className="h-8 bg-neutral-300 dark:bg-neutral-700 rounded w-3/4" />
                <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-1/4" />
                <div className="space-y-2 pt-4">
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-full" />
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-full" />
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-5/6" />
                </div>
                <div className="space-y-2 pt-2">
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-full" />
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-4/5" />
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-full" />
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-2/3" />
                </div>
                <div className="h-6 bg-neutral-300 dark:bg-neutral-700 rounded w-1/2 pt-2" />
                <div className="space-y-2">
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-full" />
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-3/4" />
                    <div className="h-4 bg-neutral-200 dark:bg-neutral-600 rounded w-full" />
                </div>
            </div>
        </div>
    );
}
