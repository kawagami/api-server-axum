CREATE INDEX IF NOT EXISTS idx_stock_day_all_stock_code_trade_date
ON stock_day_all (stock_code, trade_date DESC);
