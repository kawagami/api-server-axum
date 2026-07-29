/**
 * 後台表格的統一外框（白卡 + 圓角 + 橫向捲動）。
 * 寬度交給 admin layout 的容器，這裡不再自己 max-w / mx-auto —— 否則各頁表格寬度不一，切頁會左右跳。
 *
 * stickyHead：長清單用。表格區改成自己的捲動區（高度上限 70svh）並讓表頭黏在區塊頂端。
 * 必須成對出現 —— 沒有高度上限的容器，sticky 沒有可黏的參考（見 globals.css 的說明）。
 */
export default function AdminTableContainer({
    children,
    stickyHead = false,
}: {
    children: React.ReactNode;
    stickyHead?: boolean;
}) {
    return (
        <div className="w-full bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden">
            <div className={stickyHead ? "admin-sticky-head overflow-auto max-h-[70svh]" : "overflow-x-auto"}>
                {children}
            </div>
        </div>
    );
}
