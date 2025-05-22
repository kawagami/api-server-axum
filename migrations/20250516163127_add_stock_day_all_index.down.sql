-- Add down migration script here

DROP INDEX IF EXISTS idx_stock_day_all_stock_code;
DROP INDEX IF EXISTS idx_stock_day_all_trade_date;
