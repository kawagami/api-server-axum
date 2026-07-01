// 台灣電子發票證明聯「一維條碼（Code 39）」解析（純函式、零 DOM、可單元測試）
//
// 對齊財政部「電子發票證明聯一維及二維條碼規格說明」。
// 紙本電子發票的一維條碼內容為固定 19 碼：
//   0–4   (5)  發票期別：民國年(3) + 期末偶數月(2)，如 11502 = 民國115年、1-2月期
//   5–14  (10) 發票字軌號碼（2 大寫英文 + 8 數字）
//   15–18 (4)  隨機碼
//
// 條碼不含開立日期與金額（那些在左方 QR）。對獎只需期別，後端由 invoice_date 反推 period，
// 故這裡把 occurredAt 取「期末偶數月首日」（必落在該期別內、period 必正確），
// 使用者可在預覽表單微調確切日期（不影響對獎，只影響選擇記為支出時的記帳日）。

export const INVOICE_BARCODE_LEN = 19;

export interface ParsedInvoiceBarcode {
    invoiceNumber: string;
    period: string;       // 西元 YYYYMM（期末偶數月）
    occurredAt: string;   // 西元 YYYY-MM-DD（期末偶數月首日）
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
        occurredAt: `${year}-${mm}-01`,
    };
}
