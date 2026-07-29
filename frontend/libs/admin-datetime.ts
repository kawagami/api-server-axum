/**
 * 後台時間顯示的單一來源。
 *
 * 為什麼要收斂：後台的 server component 跑在容器裡（沒設 TZ，等於 UTC），
 * 直接 `new Date(x).toLocaleString()` 會少 8 小時；client component 又會跟著
 * 瀏覽器時區與 locale 跑，同一份資料在不同頁長得不一樣。這裡把 locale 與時區
 * 都釘死（後台不走 i18n，一律繁中 + 台北時間），server / client 兩邊結果一致。
 */

export const ADMIN_LOCALE = "zh-TW";
export const ADMIN_TIME_ZONE = "Asia/Taipei";

/** 無值 / 壞值的統一呈現 */
const DASH = "—";

const dateTime = new Intl.DateTimeFormat(ADMIN_LOCALE, {
    year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit",
    hourCycle: "h23", timeZone: ADMIN_TIME_ZONE,
});

const dateTimeSeconds = new Intl.DateTimeFormat(ADMIN_LOCALE, {
    year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit", second: "2-digit",
    hourCycle: "h23", timeZone: ADMIN_TIME_ZONE,
});

const dateOnly = new Intl.DateTimeFormat(ADMIN_LOCALE, {
    year: "numeric", month: "2-digit", day: "2-digit",
    timeZone: ADMIN_TIME_ZONE,
});

const timeOnly = new Intl.DateTimeFormat(ADMIN_LOCALE, {
    hour: "2-digit", minute: "2-digit",
    hourCycle: "h23", timeZone: ADMIN_TIME_ZONE,
});

type DateInput = string | number | Date | null | undefined;

function apply(fmt: Intl.DateTimeFormat, value: DateInput): string {
    if (value === null || value === undefined || value === "") return DASH;
    const d = value instanceof Date ? value : new Date(value);
    return Number.isNaN(d.getTime()) ? DASH : fmt.format(d);
}

/** 2026/07/29 14:03 —— 一般清單、詳情頁用 */
export function formatDateTime(value: DateInput): string {
    return apply(dateTime, value);
}

/** 2026/07/29 14:03:07 —— logs / audit logs 這種要對到秒的用 */
export function formatDateTimeSeconds(value: DateInput): string {
    return apply(dateTimeSeconds, value);
}

/** 2026/07/29 */
export function formatDate(value: DateInput): string {
    return apply(dateOnly, value);
}

/** 14:03 —— 圖表軸標這種只需要時分的用 */
export function formatTimeOfDay(value: DateInput): string {
    return apply(timeOnly, value);
}
