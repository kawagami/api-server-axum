// 台灣電子發票證明聯「左方 QR Code」解析（純函式、零 DOM、可單元測試）
//
// 對齊財政部「電子發票證明聯一維及二維條碼規格說明」v1.9（民111/2022）。
// 左方 QR 前 77 字元為定長段，之後 ':' 分隔為品目明細（選用）。
// 定長段佈局（0-index）：
//   0–9   (10) 發票字軌號碼（2 英文 + 8 數字）
//   10–16 (7)  開立日期：民國 YYYMMDD（年 3 碼）
//   17–20 (4)  隨機碼
//   21–28 (8)  銷售額（未稅，16 進位）
//   29–36 (8)  總計額（含稅，16 進位）
//   37–44 (8)  買方統編
//   45–52 (8)  賣方統編
//   53–76 (24) 加密驗證資訊（AES + base64）
//
// 兩個關鍵轉換：金額為 16 進位整數（單位：元）、日期為民國年需 +1911。

export const INVOICE_FIXED_LEN = 77;

export interface ParsedInvoice {
    invoiceNumber: string;
    occurredAt: string;       // 西元 YYYY-MM-DD
    amount: string;           // 含稅總計，整數元字串
    salesAmount: string;      // 未稅銷售額，整數元字串
    randomCode: string;
    buyerTaxId: string | null;
    sellerTaxId: string | null;
    note: string | null;      // 由品目明細組出（僅 UTF-8 編碼時）
}

function rocToIso(yyymmdd: string): string | null {
    if (!/^\d{7}$/.test(yyymmdd)) return null;
    const year = parseInt(yyymmdd.slice(0, 3), 10) + 1911;
    const mm = yyymmdd.slice(3, 5);
    const dd = yyymmdd.slice(5, 7);
    const m = parseInt(mm, 10);
    const d = parseInt(dd, 10);
    if (m < 1 || m > 12 || d < 1 || d > 31) return null;
    return `${year}-${mm}-${dd}`;
}

function normTaxId(raw: string): string | null {
    const t = raw.trim();
    if (!/^\d{8}$/.test(t) || t === '00000000') return null;
    return t;
}

// 明細段：完整品目筆數:總品目筆數:編碼方式:品名1:數量1:單價1:品名2…
// 編碼方式 0=Big5 1=UTF-8 2=Base64；只有 UTF-8 直接可讀，其餘略過避免亂碼
function extractNote(rest: string): string | null {
    if (!rest) return null;
    // 定長段與明細交界可能帶一個前導 ':'，去掉以對齊：完整筆數:總筆數:編碼:品名:數量:單價…
    const parts = rest.replace(/^:/, '').split(':');
    if (parts.length < 4) return null;
    if (parts[2] !== '1') return null; // 非 UTF-8 不嘗試解碼
    const names: string[] = [];
    for (let i = 3; i < parts.length; i += 3) {
        const name = parts[i]?.trim();
        if (name) names.push(name);
    }
    const note = names.join('、');
    return note || null;
}

/**
 * 解析左方 QR 解碼字串。格式不符（長度不足、字軌/日期/金額不合法）回傳 null。
 */
export function parseInvoiceQr(raw: string): ParsedInvoice | null {
    if (!raw) return null;
    const seg = raw.trim();
    if (seg.length < INVOICE_FIXED_LEN) return null;

    const invoiceNumber = seg.slice(0, 10).toUpperCase();
    if (!/^[A-Z]{2}\d{8}$/.test(invoiceNumber)) return null;

    const occurredAt = rocToIso(seg.slice(10, 17));
    if (!occurredAt) return null;

    const randomCode = seg.slice(17, 21);

    const sales = parseInt(seg.slice(21, 29), 16);
    const total = parseInt(seg.slice(29, 37), 16);
    if (!Number.isFinite(total) || total <= 0) return null;

    const buyerTaxId = normTaxId(seg.slice(37, 45));
    const sellerTaxId = normTaxId(seg.slice(45, 53));
    const note = extractNote(seg.slice(INVOICE_FIXED_LEN));

    return {
        invoiceNumber,
        occurredAt,
        amount: String(total),
        salesAmount: String(Number.isFinite(sales) ? sales : 0),
        randomCode,
        buyerTaxId,
        sellerTaxId,
        note,
    };
}
