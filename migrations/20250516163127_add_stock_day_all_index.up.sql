-- Add up migration script here

CREATE INDEX IF NOT EXISTS idx_stock_day_all_stock_code ON stock_day_all (stock_code);
CREATE INDEX IF NOT EXISTS idx_stock_day_all_trade_date ON stock_day_all (trade_date);
