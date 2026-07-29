import { BorderedTableSkeleton } from "@/components/loading/table-skeleton";

const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse";

export default function Loading() {
    return (
        <div className="flex flex-col gap-4">
            <div className={`h-7 w-32 ${PULSE}`} />
            <div className={`h-4 w-full max-w-xl ${PULSE}`} />
            <div className="flex flex-col sm:flex-row gap-2">
                <div className={`h-10 flex-1 ${PULSE}`} />
                <div className={`h-10 w-40 ${PULSE}`} />
            </div>
            <BorderedTableSkeleton headers={['名稱', '建立時間', '最後使用', '操作']} rows={3} />
        </div>
    );
}
