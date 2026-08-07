// 台灣電子發票證明聯「一維條碼（Code 39）」解析（純函式、零 DOM、可單元測試）
//
// 對齊財政部「電子發票證明聯一維及二維條碼規格說明」。
// 紙本電子發票的一維條碼內容為固定 19 碼：
//   0–4   (5)  發票期別：民國年(3) + 期末偶數月(2)，如 11502 = 民國115年、1-2月期
//   5–14  (10) 發票字軌號碼（2 大寫英文 + 8 數字）
//   15–18 (4)  隨機碼
//
// 條碼不含開立日期與金額（那些在左方 QR）。**期別才是條碼裡真正有的事實**，
// 登錄時直接把 period 送給後端（`POST /member/invoices` 的 period 欄）；
// 後端拿它跟使用者確認的 invoice_date 對帳，改到別期會 422，不會靜默對錯獎。
// defaultDate 只是表單日期欄的預設值（取期末偶數月首日，必落在該期別內），
// 使用者可在預覽表單微調確切日期。

export const INVOICE_BARCODE_LEN = 19;

export interface ParsedInvoiceBarcode {
    invoiceNumber: string;
    /** 西元 YYYYMM（期末偶數月）—— 條碼裡真正帶的資訊，直接送後端對獎 */
    period: string;
    /** 表單日期欄的預設值（西元 YYYY-MM-DD，期末偶數月首日）；條碼本身沒有開立日 */
    defaultDate: string;
}

/**
 * 解析一維條碼解碼字串。格式不符（長度不足、期別/字軌不合法）回傳 null。
 */
export function parseInvoiceBarcode(raw: string): ParsedInvoiceBarcode | null {
    if (!raw) return null;
    const seg = raw.trim().toUpperCase();
    if (seg.length < INVOICE_BARCODE_LEN) return null;

    const rocPeriod = seg.slice(0, 5); // YYYMM（民國）
    if (!/^\d{5}$/.test(rocPeriod)) return null;
    const year = parseInt(rocPeriod.slice(0, 3), 10) + 1911;
    const mm = rocPeriod.slice(3, 5);
    const month = parseInt(mm, 10);
    if (month < 1 || month > 12) return null;

    const invoiceNumber = seg.slice(5, 15);
    if (!/^[A-Z]{2}\d{8}$/.test(invoiceNumber)) return null;

    return {
        invoiceNumber,
        period: `${year}${mm}`,
        defaultDate: `${year}-${mm}-01`,
    };
}
