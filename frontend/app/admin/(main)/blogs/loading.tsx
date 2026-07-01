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
        <div className="w-full p-6 bg-neutral-100 dark:bg-neutral-900">
            <div className="mb-8 flex justify-center">
                <div className="h-10 w-32 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
            </div>
            <div className="bg-white dark:bg-neutral-800 shadow rounded-lg p-6">
                <ul className="divide-y divide-neutral-200 dark:divide-neutral-700">
                    {Array.from({ length: 8 }).map((_, i) => (
                        <SkeletonRow key={i} />
                    ))}
                </ul>
            </div>
        </div>
    );
}
