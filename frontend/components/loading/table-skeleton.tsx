const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse";
// 寬度循環，讓骨架看起來不死板
const WIDTHS = ["w-24", "w-40", "w-28", "w-32", "w-20", "w-36"];

// 對齊 components/admin/table.tsx 的格線外觀（後台表格只有這一套視覺）
const CELL = "border border-neutral-300 dark:border-neutral-700 px-4 py-2";

interface TableSkeletonProps {
    headers: string[];
    rows?: number;
}

function SkeletonTable({ headers, rows = 8 }: TableSkeletonProps) {
    return (
        <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
            <div className="overflow-x-auto">
                <table className="w-full border-collapse border border-neutral-200 dark:border-neutral-700 text-sm">
                    <thead>
                        <tr className="bg-neutral-100 dark:bg-neutral-800">
                            {headers.map((h, i) => (
                                <th key={`${h}-${i}`} className={`${CELL} text-left text-neutral-700 dark:text-neutral-300`}>
                                    {h}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {Array.from({ length: rows }).map((_, r) => (
                            <tr key={r}>
                                {headers.map((h, c) => (
                                    <td key={`${h}-${c}`} className={CELL}>
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

/** 只有表格的骨架。外層 padding / 寬度由 admin layout 給，這裡不要再加 */
export function BorderedTableSkeleton({ headers, rows = 8 }: TableSkeletonProps) {
    return (
        <div className="w-full flex flex-col gap-4">
            <div className={`h-7 w-32 ${PULSE}`} />
            <SkeletonTable headers={headers} rows={rows} />
        </div>
    );
}

/** 標題列 + 篩選列 + 表格的骨架（對應有篩選條的清單頁） */
export function ListTableSkeleton({ headers, rows = 10 }: TableSkeletonProps) {
    return (
        <div className="w-full flex flex-col gap-4">
            <div className="flex items-center justify-between flex-wrap gap-3">
                <div className={`h-7 w-24 ${PULSE}`} />
                <div className="flex gap-2">
                    {Array.from({ length: 4 }).map((_, i) => (
                        <div key={i} className={`h-8 w-16 ${PULSE}`} />
                    ))}
                </div>
            </div>
            <SkeletonTable headers={headers} rows={rows} />
        </div>
    );
}
