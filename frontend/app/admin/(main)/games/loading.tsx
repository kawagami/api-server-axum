export default function Loading() {
    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-6">
            <div className="max-w-4xl mx-auto flex flex-col gap-4">
                <div className="flex items-center justify-between">
                    <div className="h-7 w-32 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                    <div className="h-9 w-28 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                </div>
                <div className="h-4 w-48 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                    <table className="w-full border-collapse border border-neutral-200 dark:border-neutral-700">
                        <thead>
                            <tr className="bg-neutral-100 dark:bg-neutral-800">
                                {['遊戲', '類型', '進行中', '對局人數', '等待中', '排隊', '大廳'].map((h) => (
                                    <th key={h} className="border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-left text-neutral-700 dark:text-neutral-300">
                                        {h}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <tbody>
                            {Array.from({ length: 7 }).map((_, i) => (
                                <tr key={i}>
                                    {Array.from({ length: 7 }).map((_, j) => (
                                        <td key={j} className="border border-neutral-300 dark:border-neutral-700 px-4 py-2">
                                            <div className="h-4 w-16 bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse" />
                                        </td>
                                    ))}
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}
