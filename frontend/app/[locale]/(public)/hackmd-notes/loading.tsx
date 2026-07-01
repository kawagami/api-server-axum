export default function HackMDNotesLoading() {
    return (
        <div className="w-full h-[calc(100svh-120px)] text-center overflow-auto">
            <div className="gap-1 text-center p-2 mb-2">
                <div className="inline-block h-10 w-28 rounded-lg bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
            </div>
            <div className="h-7 w-40 mx-auto mb-4 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-6 p-4">
                {Array.from({ length: 10 }).map((_, i) => (
                    <div key={i} className="border border-neutral-300 dark:border-neutral-700 rounded-lg shadow-lg bg-white dark:bg-neutral-800 p-4">
                        <div className="h-7 w-3/4 mx-auto mb-3 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                        <div className="h-4 w-1/2 mx-auto mb-3 rounded bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                        <div className="flex gap-2 flex-wrap justify-center">
                            <div className="h-6 w-14 rounded-full bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                            <div className="h-6 w-16 rounded-full bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
                        </div>
                    </div>
                ))}
            </div>
        </div>
    );
}
