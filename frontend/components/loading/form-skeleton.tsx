const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded-sm animate-pulse";

/**
 * 後台表單／設定頁的共用骨架（標題 + 數個「說明 + 輸入列」卡片）。
 * 寬度刻意跟表單頁一致（max-w-2xl 靠左），否則載入完成會橫向跳版。
 */
export default function FormSkeleton({ fields = 4, cards = false }: { fields?: number; cards?: boolean }) {
    return (
        <div className="w-full max-w-2xl flex flex-col gap-4">
            <div className={`h-7 w-32 ${PULSE}`} />
            <div className="flex flex-col gap-3">
                {Array.from({ length: fields }).map((_, i) => (
                    <div
                        key={i}
                        className={
                            cards
                                ? "bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4 flex flex-col gap-2"
                                : "flex flex-col gap-2"
                        }
                    >
                        <div className={`h-4 w-40 ${PULSE}`} />
                        <div className={`h-10 w-full ${PULSE}`} />
                    </div>
                ))}
            </div>
        </div>
    );
}
