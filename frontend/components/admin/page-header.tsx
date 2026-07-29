/**
 * 後台頁面標題的單一來源（h1 規格、描述、右側動作區）。
 * 每頁一個、只放這一顆 h1；統計數字不要用 h1 冒充標題。
 * 本身不帶外距，放在頁面的 `flex flex-col gap-4` 容器裡由容器控制間距。
 */
export default function PageHeader({
    title,
    description,
    actions,
}: {
    title: React.ReactNode;
    description?: React.ReactNode;
    actions?: React.ReactNode;
}) {
    return (
        <div className="flex flex-wrap items-start justify-between gap-3">
            <div className="min-w-0">
                <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">
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
