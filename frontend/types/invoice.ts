// 統一發票登錄 + 對獎（解耦於記帳，走 POST /member/invoices）
export type InvoiceSource = 'qr' | 'barcode' | 'manual';

// 中獎獎別：special 特別獎、grand 特獎、first 頭獎、second~sixth 二~六獎、additional_sixth 增開六獎
export type PrizeTier =
  | 'special'
  | 'grand'
  | 'first'
  | 'second'
  | 'third'
  | 'fourth'
  | 'fifth'
  | 'sixth'
  | 'additional_sixth';

export interface Invoice {
  id: string;
  member_id: number;
  invoice_number: string;
  invoice_date: string; // 西元 YYYY-MM-DD
  period: string; // 對獎期別 key（期末偶數月 YYYYMM），後端算好
  amount: string | null; // 十進位字串，可為 null
  seller_tax_id: string | null;
  source: InvoiceSource;
  ledger_entry_id: string | null; // record_as_expense 時連結的記帳 id
  lottery_checked: boolean; // 該期是否已開獎並對過
  prize_tier: PrizeTier | null; // checked=true 且為 null = 確定未中
  notified_at: string | null;
  created_at: string;
  updated_at: string;
}

// 登錄發票（POST /member/invoices）
export interface InvoiceInput {
  invoice_number: string;
  invoice_date: string;
  amount?: string | null;
  seller_tax_id?: string | null;
  source: InvoiceSource;
  record_as_expense?: boolean; // true 時同時建一筆 ledger 支出並連結
  category?: string; // record_as_expense=true 時用，省略 → other
  note?: string | null;
}

export interface InvoiceListParams {
  period?: string; // YYYYMM
  won?: boolean; // true=只看中獎、false=只看未中、省略=全部
  page?: number;
  per_page?: number;
}

// 某期統一發票中獎號碼（GET /member/invoices/draws，一期一筆；財政部公布，對中末幾碼即中）
export interface InvoiceDraw {
  period: string; // YYYYMM（期末偶數月）
  special: string | null; // 特別獎 8 碼
  grand: string | null; // 特獎 8 碼
  first: string[]; // 頭獎 8 碼（通常 3 組）
  additional: string[]; // 增開六獎 3 碼（0~N 組）
}

export interface InvoiceDrawParams {
  period?: string; // 指定期別；省略回近期各期
  limit?: number; // 回傳期數（預設 6、上限 24）
}
