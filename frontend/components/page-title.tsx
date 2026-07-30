/**
 * 前台頁面標題的單一來源（對應後台的 components/admin/page-header.tsx）。
 * 一頁一顆 h1，字級規格只有兩種：
 *   default  靠左，一般內容頁／會員功能頁
 *   hero     置中大標，首頁與各 hub 頁（tools / games / about）
 * 本身不帶外距，放在頁面的 `flex flex-col gap-*` 容器裡由容器控制間距。
 */
export default function PageTitle({
    title,
    description,
    actions,
    variant = "default",
}: {
    title: React.ReactNode;
    description?: React.ReactNode;
    actions?: React.ReactNode;
    variant?: "default" | "hero";
}) {
    if (variant === "hero") {
        return (
            <div className="text-center">
                <h1 className="text-3xl sm:text-4xl font-bold text-neutral-800 dark:text-neutral-100">
                    {title}
                </h1>
                {description && (
                    <p className="mx-auto mt-3 max-w-2xl text-base sm:text-lg text-neutral-600 dark:text-neutral-300">
                        {description}
                    </p>
                )}
                {actions && (
                    <div className="mt-4 flex flex-wrap items-center justify-center gap-2">{actions}</div>
                )}
            </div>
        );
    }

    return (
        <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="min-w-0">
                <h1 className="text-2xl font-bold text-neutral-800 dark:text-neutral-100">
                    {title}
                </h1>
                {description && (
                    <p className="mt-1 text-sm text-neutral-500 dark:text-neutral-400">
                        {description}
                    </p>
                )}
            </div>
            {actions && <div className="flex flex-wrap items-center gap-2">{actions}</div>}
        </div>
    );
}
