// Ledger（記帳）
export type LedgerKind = 'income' | 'expense';

export type LedgerSource = 'manual' | 'invoice_qr';

export interface LedgerEntry {
  id: string;
  member_id: number;
  kind: LedgerKind;
  amount: string; // 十進位字串，避免浮點誤差
  category: string;
  note: string | null;
  occurred_at: string; // YYYY-MM-DD
  // 掃發票匯入會帶這三欄；手動記帳 invoice_number/seller_tax_id 為 null、source='manual'
  invoice_number?: string | null;
  seller_tax_id?: string | null;
  source?: LedgerSource;
  created_at: string;
  updated_at: string;
}

export interface LedgerInput {
  kind: LedgerKind;
  amount: string;
  category: string;
  note?: string | null;
  occurred_at: string;
}

export interface LedgerCategoryOption {
  value: string;
  label: string;
}

export interface LedgerCategories {
  income: LedgerCategoryOption[];
  expense: LedgerCategoryOption[];
}

export interface LedgerCategoryTotal {
  kind: LedgerKind;
  category: string;
  total: string;
}

export interface LedgerMonthly {
  month: string; // YYYY-MM
  income: string;
  expense: string;
}

export interface LedgerSummary {
  total_income: string;
  total_expense: string;
  balance: string;
  by_category: LedgerCategoryTotal[];
  monthly: LedgerMonthly[];
}
