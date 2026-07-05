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
}
