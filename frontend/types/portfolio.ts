// Portfolio
export interface HistoryRecord {
  date: string;
  close: number;
  adjusted_cost: number;
  pnl: number;
  pnl_pct: number;
}

export interface PortfolioEntry {
  id: string;
  member_id: number;
  stock_code: string;
  buy_date: string;
  cost_per_share: number;
  shares: number;
  created_at: string;
  updated_at: string;
}

export interface PortfolioEntryInput {
  stock_code: string;
  buy_date: string;
  cost_per_share: number;
  shares: number;
}

export interface PortfolioSummaryEntry extends PortfolioEntry {
  stock_name: string;
  current_price: number | null;
  current_value: number | null;
  pnl: number | null;
  pnl_pct: number | null;
  changes: PeriodChanges;
}

export type PeriodKey = 'day' | 'week' | 'month';

/** 一筆持股在某期間的增減。基準價後端已還原期間內的除權息。 */
export interface PeriodChange {
  /** 實際比較的交易日（休市時會早於目標日） */
  base_date: string;
  base_close: number;
  /** 每股增減 */
  change: number;
  change_pct: number;
  /** 這筆持股的市值增減 */
  value_change: number;
}

/** null = 該期間沒有可用的基準日（新持股，或行情還沒補齊） */
export type PeriodChanges = Record<PeriodKey, PeriodChange | null>;
