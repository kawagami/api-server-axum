// Stock
export interface StockDayAll {
  id: number;
  trade_date: string;
  stock_code: string;
  stock_name: string;
  trade_volume: number | null;
  trade_amount: number | null;
  open_price: number | null;
  high_price: number | null;
  low_price: number | null;
  close_price: number | null;
  price_change: number | null;
  transaction_count: number | null;
}

export interface StockBuybackPeriod {
  stock_no: string;
  start_date: string;
  end_date: string;
}

export interface StockChange {
  id: string | number;
  stock_no: string;
  stock_name: string;
  status: string;
  start_date: string;
  start_price: number;
  end_date: string;
  end_price: number;
  change: number;
}

export interface StockChangePaginatedResponse {
  data: StockChange[];
  total: number;
}
