/** 後台清單載入失敗的統一文案（後台不走 i18n）；公開頁請用各自 namespace 的 loadFailed */
export const LOAD_FAILED = '載入失敗，請稍後再試';

/** 後台刪除失敗的統一文案（不要用 window.alert） */
export const DELETE_FAILED = '刪除失敗，請稍後再試';

/**
 * 後台頁面層級的錯誤呈現。欄位層級的驗證訊息請留在欄位旁的 inline <p>，
 * 這裡專門放「這一頁的資料/操作失敗了」這種訊息。
 * role="alert" 讓螢幕閱讀器在訊息出現時即時朗讀。
 */
export default function ErrorBanner({ message }: { message: string | null }) {
    if (!message) return null;
    return (
        <div
            role="alert"
            className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-sm p-3 text-red-700 dark:text-red-400 text-sm"
        >
            {message}
        </div>
    );
}
