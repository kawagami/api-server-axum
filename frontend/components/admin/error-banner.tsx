/** 後台清單載入失敗的統一文案(後台不走 i18n);公開頁請用各自 namespace 的 loadFailed */
export const LOAD_FAILED = '載入失敗,請稍後再試';

export default function ErrorBanner({ message }: { message: string | null }) {
    if (!message) return null;
    return (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 text-red-700 dark:text-red-400 text-sm">
            {message}
        </div>
    );
}
