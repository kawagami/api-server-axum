/**
 * 後台表格的統一外框（白卡 + 圓角 + 橫向捲動）。
 * 寬度交給 admin layout 的容器，這裡不再自己 max-w / mx-auto —— 否則各頁表格寬度不一，切頁會左右跳。
 *
 * stickyHead：長清單用。表格區改成自己的捲動區（高度上限 70svh）並讓表頭黏在區塊頂端。
 * 必須成對出現 —— 沒有高度上限的容器，sticky 沒有可黏的參考（見 globals.css 的說明）。
 *
 * fill：把「高度上限 70svh」換成「吃掉版面剩下的高度」。70svh 是猜的，內容只要比一屏
 * 高一點點，admin layout 的 overflow-auto 就會跟表格自己的捲動區各長一條捲軸、兩條並排。
 * fill 讓整頁剛好塞滿一屏，只有表格會捲。
 * 代價與前提：呼叫端從 page root 到這裡的每一層都要是 `flex min-h-0 flex-1 flex-col`
 * （少一層 min-h-0 就會被內容撐開、退回兩條捲軸），且清單很短時卡片仍是滿高的空白。
 */
export default function AdminTableContainer({
    children,
    stickyHead = false,
    fill = false,
}: {
    children: React.ReactNode;
    stickyHead?: boolean;
    fill?: boolean;
}) {
    const scrollArea = fill
        ? `overflow-auto min-h-0 flex-1${stickyHead ? " admin-sticky-head" : ""}`
        : stickyHead
          ? "admin-sticky-head overflow-auto max-h-[70svh]"
          : "overflow-x-auto";
    return (
        <div
            className={`w-full bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden${
                fill ? " flex min-h-0 flex-1 flex-col" : ""
            }`}
        >
            <div className={scrollArea}>{children}</div>
        </div>
    );
}
