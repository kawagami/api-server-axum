/**
 * 條件式 className 組合：把 falsy 濾掉再用空白接起來。
 *
 * 刻意**不裝** `clsx` / `tailwind-merge` —— 全庫的用法都只是
 * 「基礎樣式 + 幾段條件樣式」的字串接合，那正是這 3 行做的事。
 *
 * ⚠️ 這支**不會**處理 Tailwind 的衝突覆寫（`px-2` 與 `px-4` 同時出現時，
 * 贏的是 CSS 產生順序、不是參數順序）。要覆寫基礎樣式的某個屬性，
 * 就把該屬性從基礎樣式拿掉，不要疊上去賭順序。
 */
export function cn(...parts: Array<string | false | null | undefined>): string {
    return parts.filter(Boolean).join(" ");
}
