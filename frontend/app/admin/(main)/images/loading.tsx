const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse";

export default function Loading() {
    return (
        <div className="w-full flex flex-col gap-4">
            <div className={`h-7 w-20 ${PULSE}`} />
            <div className={`h-4 w-80 max-w-full ${PULSE}`} />
            <div className="flex flex-col items-center gap-4 py-4">
                <div className={`h-10 w-28 ${PULSE}`} />
            </div>
            <div className="flex flex-wrap items-center gap-2">
                {Array.from({ length: 3 }).map((_, i) => (
                    <div key={i} className={`h-8 w-20 rounded-full ${PULSE}`} />
                ))}
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {Array.from({ length: 6 }).map((_, i) => (
                    <div key={i} className="bg-white dark:bg-neutral-900 p-4 rounded-sm shadow-md flex flex-col items-center gap-3">
                        <div className={`w-[150px] h-[150px] rounded-lg ${PULSE}`} />
                        <div className={`h-4 w-20 ${PULSE}`} />
                        <div className={`h-8 w-20 ${PULSE}`} />
                    </div>
                ))}
            </div>
        </div>
    );
}
