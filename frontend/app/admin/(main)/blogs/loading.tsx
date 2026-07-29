function SkeletonRow() {
    return (
        <li className="flex items-center justify-between p-4">
            <div className="h-4 w-64 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
            <div className="flex space-x-2">
                <div className="h-9 w-16 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
                <div className="h-9 w-16 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
            </div>
        </li>
    );
}

export default function Loading() {
    return (
        <div className="w-full flex flex-col gap-4">
            <div className="flex items-center justify-between gap-3">
                <div className="h-7 w-24 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                <div className="h-10 w-28 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
            </div>
            <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg p-4 sm:p-6">
                <ul className="divide-y divide-neutral-200 dark:divide-neutral-700">
                    {Array.from({ length: 8 }).map((_, i) => (
                        <SkeletonRow key={i} />
                    ))}
                </ul>
            </div>
        </div>
    );
}
