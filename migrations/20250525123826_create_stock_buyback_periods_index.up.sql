-- Add up migration script here
-- Add up migration script here

CREATE INDEX IF NOT EXISTS idx_stock_buyback_periods_stock_no ON stock_buyback_periods (stock_no);
CREATE INDEX IF NOT EXISTS idx_stock_buyback_periods_start_date ON stock_buyback_periods (start_date);
CREATE INDEX IF NOT EXISTS idx_stock_buyback_periods_end_date ON stock_buyback_periods (end_date);
