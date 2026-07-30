import PageShell from "@/components/page-shell";

const PULSE = "bg-neutral-200 dark:bg-neutral-700 rounded animate-pulse";

/**
 * 前台資料頁的載入骨架（會員功能頁全部走這支）。
 * 容器規格必須與對應 page.tsx 一致（同 width、同 gap-6），否則載完會跳版。
 *
 * variant：
 *   list  清單頁（發票 / 樂透 / 記帳 / 持股）
 *   cards 卡片網格（dashboard）
 *   form  單張表單卡（設定 / 個人資料）
 */
export default function PublicPageSkeleton({
    width = "content",
    nav = false,
    variant = "list",
    rows = 5,
}: {
    width?: "form" | "content" | "wide";
    nav?: boolean;
    variant?: "list" | "cards" | "form";
    rows?: number;
}) {
    return (
        <PageShell width={width} className="flex flex-col gap-6">
            <div className={`h-8 w-40 ${PULSE}`} />

            {nav && (
                <div className="flex flex-wrap gap-2">
                    {Array.from({ length: 5 }).map((_, i) => (
                        <div key={i} className={`h-8 w-20 rounded-full ${PULSE}`} />
                    ))}
                </div>
            )}

            {variant === "cards" ? (
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {Array.from({ length: rows }).map((_, i) => (
                        <div key={i} className={`h-28 rounded-xl ${PULSE}`} />
                    ))}
                </div>
            ) : variant === "form" ? (
                <div className="bg-white dark:bg-neutral-800 rounded-xl p-6 shadow flex flex-col gap-4">
                    {Array.from({ length: rows }).map((_, i) => (
                        <div key={i} className={`h-10 ${PULSE}`} />
                    ))}
                </div>
            ) : (
                <div className="flex flex-col gap-3">
                    {Array.from({ length: rows }).map((_, i) => (
                        <div key={i} className={`h-16 rounded-lg ${PULSE}`} />
                    ))}
                </div>
            )}
        </PageShell>
    );
}
