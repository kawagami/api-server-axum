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
  /** 前一交易日收盤價，後端已還原期間內的除權息；只有一天行情時為 null */
  prev_close: number | null;
  day_change: number | null;
  day_change_pct: number | null;
  /** 這筆持股的當日市值增減 */
  day_value_change: number | null;
}
