export default function Loading() {
    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto flex flex-col items-center">
            <div className="h-10 w-64 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse my-8" />
            <div className="w-full flex justify-center gap-4 flex-wrap">
                {Array.from({ length: 7 }).map((_, i) => (
                    <div key={i} className="h-12 w-36 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
                ))}
            </div>
        </div>
    );
}
