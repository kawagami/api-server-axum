import type { PaginatedResponse } from './pagination';

// Stock
// 價格欄位後端是 `rust_decimal::Decimal`，序列化成**字串**（`"123.45"`）以保留精度，
// 不是 number —— 要算數先 Number()
export interface StockDayAll {
  id: number;
  trade_date: string;
  stock_code: string;
  stock_name: string;
  trade_volume: number | null;
  trade_amount: number | null;
  open_price: string | null;
  high_price: string | null;
  low_price: string | null;
  close_price: string | null;
  price_change: string | null;
  transaction_count: number | null;
}

export interface StockBuybackPeriod {
  stock_no: string;
  start_date: string;
  end_date: string;
}

// 後端 `structs/stocks.rs::StockChange`：除了代號與日期，其餘欄位都可能為 null
// （剛寫入 pending、行情還沒抓到）
export interface StockChange {
  id: number | null;
  stock_no: string;
  stock_name: string | null;
  status: string | null;
  start_date: string;
  start_price: number | null;
  end_date: string;
  end_price: number | null;
  change: number | null;
}

// `GET /admin/stocks/buyback_price_gaps`（後端 `StockBuybackMoreInfo`）。
// 開始日沒有收盤價時 price_on_start_date / diff / diff_percent 為 null
export interface StockBuybackPriceGap {
  stock_no: string;
  stock_name: string;
  start_date: string;
  end_date: string;
  price_on_start_date: number | null;
  latest_price: number | null;
  diff: number | null;
  diff_percent: number | null;
}

export type StockChangePaginatedResponse = PaginatedResponse<StockChange>;
