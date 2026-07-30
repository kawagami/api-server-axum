/**
 * 前台頁面容器的單一來源：內容寬度、左右 padding、上下 padding 都由這裡給。
 * 頁面不要再自己寫 `max-w-*` / `px-4` / `py-8`，否則各頁寬度不一，切頁時內容左右跳。
 *
 * 三種寬度，其餘一律不開：
 *   form    表單／設定／單欄詳情頁
 *   content 一般內容頁（預設）
 *   wide    卡片網格的 hub 頁（首頁 / tools / games / dashboard）
 *
 * 捲動一律交給 body（header 是 sticky），頁面不要自己開 `h-[calc(100svh-120px)] overflow-auto`
 * ——那會變成頁內捲動、header 反而不動，與其他頁的體感不一致。
 * 例外只有遊戲：棋盤要塞滿一屏不捲，那些頁維持自己的固定高度。
 */
const WIDTHS = {
    form: "max-w-2xl",
    content: "max-w-4xl",
    wide: "max-w-5xl",
} as const;

export default function PageShell({
    width = "content",
    className = "",
    children,
}: {
    width?: keyof typeof WIDTHS;
    className?: string;
    children: React.ReactNode;
}) {
    return (
        <div className={`mx-auto w-full ${WIDTHS[width]} px-4 py-6 sm:py-8 ${className}`}>
            {children}
        </div>
    );
}
