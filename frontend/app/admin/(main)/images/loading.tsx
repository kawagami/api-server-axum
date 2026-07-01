export default function Loading() {
    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-6">
            <div className="container mx-auto p-6">
                <div className="flex flex-col justify-center items-center pt-4 gap-4">
                    <div className="h-8 w-48 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                    <div className="h-10 w-28 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mt-8">
                    {Array.from({ length: 6 }).map((_, i) => (
                        <div key={i} className="bg-white dark:bg-neutral-800 p-4 rounded shadow-md flex flex-col items-center gap-3">
                            <div className="w-[150px] h-[150px] bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
                            <div className="h-4 w-20 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                            <div className="h-8 w-20 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
