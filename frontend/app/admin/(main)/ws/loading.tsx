// 容器規格要跟 page.tsx 對齊（外層 layout 已給 padding），否則載入完成會跳版
export default function Loading() {
    return (
        <div className="w-full">
            <div className="flex flex-col gap-8">
                <section className="flex flex-col gap-3">
                    <div className="flex items-center justify-between flex-wrap gap-3">
                        <div className="h-7 w-40 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                        <div className="h-8 w-64 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                    </div>
                    <div className="h-4 w-56 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                    <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                        <table className="w-full border-collapse border border-neutral-200 dark:border-neutral-700">
                            <thead>
                                <tr className="bg-neutral-100 dark:bg-neutral-800">
                                    {['真實 IP', '連線位址', '使用者', '裝置', '連線時長', '操作'].map((h, i) => (
                                        <th
                                            key={h}
                                            className={`border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 ${i === 1 ? 'hidden md:table-cell' : ''} ${i === 3 ? 'hidden sm:table-cell' : ''}`}
                                        >
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {Array.from({ length: 4 }).map((_, row) => (
                                    <tr key={row}>
                                        {[28, 32, 40, 24, 20, 16].map((w, i) => (
                                            <td
                                                key={i}
                                                className={`border border-neutral-300 dark:border-neutral-700 px-4 py-2 ${i === 1 ? 'hidden md:table-cell' : ''} ${i === 3 ? 'hidden sm:table-cell' : ''}`}
                                            >
                                                <div className="h-4 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" style={{ width: `${w * 4}px` }} />
                                            </td>
                                        ))}
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </section>
                <section>
                    <div className="h-7 w-52 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse mb-4" />
                    <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg p-6 flex flex-col gap-3">
                        <div className="h-4 w-20 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                        <div className="h-7 w-44 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                        <div className="h-4 w-20 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                        <div className="h-20 w-full bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                        <div className="h-10 w-20 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                    </div>
                </section>
            </div>
        </div>
    );
}
