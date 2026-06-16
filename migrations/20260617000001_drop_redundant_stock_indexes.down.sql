CREATE INDEX IF NOT EXISTS idx_stock_day_all_stock_code ON stock_day_all (stock_code);
CREATE INDEX IF NOT EXISTS idx_stock_day_all_trade_date ON stock_day_all (trade_date);
CREATE INDEX IF NOT EXISTS idx_stock_buyback_periods_stock_no ON stock_buyback_periods (stock_no);
