// 單字闖關的純常數(server 的 load.ts 與 client 的 vocab-client.tsx 共用,
// 所以這支不能 import 任何 server-only 或 client-only 模組)

/** 答後回饋停留毫秒;使用者按 Enter / 點畫面可提前跳過 */
export const FEEDBACK_MS = 1400;

/** 限時模式可選時長(分鐘) */
export const DURATIONS = [3, 5, 10] as const;

/** 錯題本一次載入幾筆(後端上限 100) */
export const MISTAKE_PAGE_SIZE = 20;

/** 錯題本搜尋的 debounce 毫秒 */
export const SEARCH_DEBOUNCE_MS = 300;
