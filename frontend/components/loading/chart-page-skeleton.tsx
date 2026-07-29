const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse";

/**
 * 後台圖表頁（到訪統計 / 系統指標）的共用骨架：
 * 標題 + 區間切換 + 數張數值卡 + 數張圖表卡。
 */
export default function ChartPageSkeleton({ cards = 2, charts = 1 }: { cards?: number; charts?: number }) {
    return (
        <div className="flex flex-col gap-6">
            <div className="flex flex-wrap items-center justify-between gap-3">
                <div className={`h-7 w-32 ${PULSE}`} />
                <div className={`h-9 w-52 ${PULSE}`} />
            </div>

            <section className={`grid grid-cols-1 gap-4 ${cards > 2 ? "sm:grid-cols-2 lg:grid-cols-4" : "sm:grid-cols-2"}`}>
                {Array.from({ length: cards }).map((_, i) => (
                    <div
                        key={i}
                        className="bg-white dark:bg-neutral-900 rounded-lg shadow border border-neutral-200 dark:border-neutral-700 p-5 flex flex-col gap-2"
                    >
                        <div className={`h-4 w-24 ${PULSE}`} />
                        <div className={`h-8 w-20 ${PULSE}`} />
                        <div className={`h-3 w-32 ${PULSE}`} />
                    </div>
                ))}
            </section>

            {Array.from({ length: charts }).map((_, i) => (
                <section
                    key={i}
                    className="bg-white dark:bg-neutral-900 rounded-lg shadow border border-neutral-200 dark:border-neutral-700 p-5 flex flex-col gap-3"
                >
                    <div className={`h-4 w-28 ${PULSE}`} />
                    <div className={`h-48 w-full ${PULSE}`} />
                </section>
            ))}
        </div>
    );
}
