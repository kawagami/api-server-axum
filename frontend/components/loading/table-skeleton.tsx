const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse";
// 寬度循環，讓骨架看起來不死板
const WIDTHS = ["w-24", "w-40", "w-28", "w-32", "w-20", "w-36"];

interface TableSkeletonProps {
    headers: string[];
    rows?: number;
}

/** 對應 AdminTable（bordered）的表格骨架，含 AdminTableContainer 外框 */
export function BorderedTableSkeleton({ headers, rows = 8 }: TableSkeletonProps) {
    return (
        <div className="w-full h-[calc(100svh-180px)] overflow-auto p-6">
            <div className="max-w-4xl mx-auto bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                <table className="w-full border-collapse border border-neutral-200 dark:border-neutral-700">
                    <thead>
                        <tr className="bg-neutral-100 dark:bg-neutral-800">
                            {headers.map((h) => (
                                <th key={h} className="border border-neutral-300 dark:border-neutral-700 px-4 py-2 text-left text-neutral-700 dark:text-neutral-300">
                                    {h}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {Array.from({ length: rows }).map((_, r) => (
                            <tr key={r}>
                                {headers.map((h, c) => (
                                    <td key={h} className="border border-neutral-300 dark:border-neutral-700 px-4 py-2">
                                        <div className={`h-4 ${WIDTHS[(r + c) % WIDTHS.length]} ${PULSE}`} />
                                    </td>
                                ))}
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}

/** 對應 logs / audit_logs 的 borderless 列表骨架（標題 + 篩選列 + 表格） */
export function ListTableSkeleton({ headers, rows = 10 }: TableSkeletonProps) {
    return (
        <div className="w-full">
            <div className="flex flex-col gap-4">
                <div className="flex items-center justify-between flex-wrap gap-3">
                    <div className={`h-7 w-24 ${PULSE}`} />
                    <div className="flex gap-2">
                        {Array.from({ length: 4 }).map((_, i) => (
                            <div key={i} className={`h-8 w-16 ${PULSE}`} />
                        ))}
                    </div>
                </div>
                <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
                    <table className="w-full border-collapse text-sm">
                        <thead>
                            <tr className="bg-neutral-100 dark:bg-neutral-800">
                                {headers.map((h) => (
                                    <th key={h} className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700">
                                        {h}
                                    </th>
                                ))}
                            </tr>
                        </thead>
                        <tbody>
                            {Array.from({ length: rows }).map((_, r) => (
                                <tr key={r} className="border-b border-neutral-100 dark:border-neutral-800">
                                    {headers.map((h, c) => (
                                        <td key={h} className="px-4 py-2">
                                            <div className={`h-4 ${WIDTHS[(r + c) % WIDTHS.length]} ${PULSE}`} />
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
