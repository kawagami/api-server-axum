const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse";

// 對應 page.tsx 的儀表板版面（標題 + 統計卡 + 分組快速入口），形狀對不上載入完會跳版
export default function Loading() {
    return (
        <div className="flex flex-col gap-8">
            <div className={`h-8 w-48 ${PULSE}`} />

            <section className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4">
                {Array.from({ length: 5 }).map((_, i) => (
                    <div
                        key={i}
                        className="flex flex-col gap-2 p-5 bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-700"
                    >
                        <div className={`h-4 w-16 ${PULSE}`} />
                        <div className={`h-8 w-12 ${PULSE}`} />
                        <div className={`h-3 w-20 ${PULSE}`} />
                    </div>
                ))}
            </section>

            <section className="flex flex-col gap-6">
                {Array.from({ length: 3 }).map((_, g) => (
                    <div key={g} className="flex flex-col gap-3">
                        <div className={`h-5 w-24 ${PULSE}`} />
                        <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-3">
                            {Array.from({ length: 4 }).map((_, i) => (
                                <div key={i} className={`h-11 ${PULSE}`} />
                            ))}
                        </div>
                    </div>
                ))}
            </section>
        </div>
    );
}
