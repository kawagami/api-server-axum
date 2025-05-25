-- Add down migration script here

DROP INDEX IF EXISTS idx_stock_buyback_periods_stock_no;
DROP INDEX IF EXISTS idx_stock_buyback_periods_start_date;
DROP INDEX IF EXISTS idx_stock_buyback_periods_end_date;
