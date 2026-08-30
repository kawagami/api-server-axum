/**
 * 輸入欄位樣式的單一來源（對應 `libs/badge-styles.ts` 的角色）。
 * 收斂前是 15 個檔案各自宣告 `const inputClass = "..."`，其中兩組各 4–6 份逐字相同。
 *
 * 只有兩個色系，因為前後台的卡片底色不同：
 *   後台卡片 `bg-white dark:bg-neutral-900` → 欄位同底色，靠 border 分界（明暗一致）
 *   前台卡片 `bg-white dark:bg-neutral-800` → 欄位 `dark:bg-neutral-700` 才浮得出來
 *
 * 尺寸差異（`w-full`、`text-sm`、`font-mono`）由呼叫端用 `cn()` 疊，不再開新常數。
 */

/** 後台表單欄位。要撐滿的呼叫端自己加 `w-full` */
export const ADMIN_INPUT =
    "px-3 py-2 rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100";

/** 後台清單頁篩選列的窄欄位（比 ADMIN_INPUT 矮一階、圓角小一階） */
export const ADMIN_FILTER_INPUT =
    "px-2 py-1.5 text-sm rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100";

/**
 * 前台表單欄位（**卡片內**用）。淺色不指定底色，靠卡片的白底透出來。
 * 不在卡片上的表單（`contact/contact-form.tsx`）底色要跟著頁面漸層走，不適用這支。
 */
export const PUBLIC_INPUT =
    "border rounded-sm px-3 py-2 text-sm dark:bg-neutral-700 dark:border-neutral-600";
