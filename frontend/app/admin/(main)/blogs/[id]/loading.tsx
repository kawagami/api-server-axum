export default function Loading() {
    return (
        // 形狀對齊 page.tsx：PageHeader 一行 + 滿版編輯器（高度公式兩邊要一致）
        <div className="flex flex-col gap-4">
            <div className="h-7 w-24 bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
            <div className="h-[calc(100svh-224px)] w-full flex flex-col">
                <div className="flex justify-evenly m-4 gap-2">
                    <div className="h-10 w-24 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
                    <div className="h-10 w-24 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
                    <div className="h-10 w-24 bg-neutral-200 dark:bg-neutral-700 rounded-lg animate-pulse" />
                </div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 px-4 flex-1 min-h-0">
                    <div className="h-full bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                    <div className="h-full bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse" />
                </div>
            </div>
        </div>
    );
}
