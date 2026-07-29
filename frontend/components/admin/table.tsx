const cellBorder = "border border-neutral-300 dark:border-neutral-700 px-4 py-2";

/**
 * 後台表格的統一外觀（有格線）。全後台表格一律用這組，不要自己手寫 cell class，
 * 也不要另外做一套「只有底線」的表格 —— 兩套視覺語言會讓頁面之間看起來不像同一個後台。
 *
 * 文字顏色刻意掛在 <table> 而不是每個 <td>：靠繼承才讓個別 cell 的
 * `text-neutral-500` / `text-red-600` 這類覆寫真的生效（同特異度時 Tailwind
 * 的輸出順序會讓 td 上的預設色反過來蓋掉呼叫端）。
 */
export function AdminTable({ className = "", ...props }: React.ComponentProps<"table">) {
    return (
        <table
            className={`w-full border-collapse border border-neutral-200 dark:border-neutral-700 text-neutral-900 dark:text-neutral-100 ${className}`}
            {...props}
        />
    );
}

export function AdminHeadRow({ className = "", ...props }: React.ComponentProps<"tr">) {
    return <tr className={`bg-neutral-100 dark:bg-neutral-800 ${className}`} {...props} />;
}

/**
 * tone：整列要帶底色時（如 log level 的紅／黃列）用它取代預設 hover，
 * 否則兩組 `hover:bg-*` 會互搶、實際生效的是 Tailwind 輸出順序在後面的那個。
 */
export function AdminRow({ className = "", tone, ...props }: React.ComponentProps<"tr"> & { tone?: string }) {
    return (
        <tr
            className={`${tone ?? "hover:bg-neutral-50 dark:hover:bg-neutral-800"} ${className}`}
            {...props}
        />
    );
}

export function AdminTh({ className = "", ...props }: React.ComponentProps<"th">) {
    return <th className={`${cellBorder} text-left text-neutral-700 dark:text-neutral-300 ${className}`} {...props} />;
}

export function AdminTd({ className = "", ...props }: React.ComponentProps<"td">) {
    return <td className={`${cellBorder} ${className}`} {...props} />;
}

/**
 * 清單的統一空狀態：表格永遠不要只剩表頭空殼（看起來像壞了）。
 * 載入中與「真的沒資料」文案由呼叫端決定。
 */
export function AdminEmptyRow({
    colSpan,
    children = "目前沒有資料",
}: {
    colSpan: number;
    children?: React.ReactNode;
}) {
    return (
        <tr>
            <td
                colSpan={colSpan}
                className="border border-neutral-300 dark:border-neutral-700 px-4 py-8 text-center text-neutral-500 dark:text-neutral-400"
            >
                {children}
            </td>
        </tr>
    );
}
