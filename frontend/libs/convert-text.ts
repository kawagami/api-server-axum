/**
 * 簡繁轉換頁的共用契約（client / server action 兩邊都吃，所以不能放在
 * `api/tools.ts` —— 那是 `"use server"` 檔，只能 export async function）。
 */

export type ConversionDirection = "s2t" | "t2s";

/**
 * 輸入上限，對齊 backend `routes/tools.rs` 的 `CONVERT_TEXT_MAX_BYTES`。
 * 改一邊要同步另一邊，否則前端擋不住的內容會變成後端的 422。
 */
export const CONVERT_TEXT_MAX_BYTES = 256 * 1024;
export const CONVERT_TEXT_MAX_KB = CONVERT_TEXT_MAX_BYTES / 1024;

/**
 * UTF-8 位元組長度。後端量的是 `String::len()`（bytes），所以前端也必須量 bytes ——
 * 中文一字 3 bytes，拿 `str.length`（UTF-16 code unit）當上限比對會放過將近 3 倍大的
 * 內容，白跑一趟請求再吃 422。
 *
 * 不用 `TextEncoder().encode(s).length` / `new Blob([s]).size`：這個值要跟著每次輸入
 * 重算，那兩種寫法每次都配置一份最大 256KB 的緩衝區只為了拿它的長度。
 */
export function utf8ByteLength(text: string): number {
    let bytes = 0;
    for (let i = 0; i < text.length; i++) {
        const code = text.charCodeAt(i);
        if (code < 0x80) {
            bytes += 1;
            continue;
        }
        if (code < 0x800) {
            bytes += 2;
            continue;
        }
        // surrogate pair（emoji 等 BMP 外字元）= UTF-8 4 bytes，兩個 code unit 一起算；
        // 落單的 surrogate 走下面的 3 bytes（等同編碼器的 U+FFFD 替代）
        if (code >= 0xd800 && code <= 0xdbff && i + 1 < text.length) {
            const next = text.charCodeAt(i + 1);
            if (next >= 0xdc00 && next <= 0xdfff) {
                bytes += 4;
                i++;
                continue;
            }
        }
        bytes += 3;
    }
    return bytes;
}
