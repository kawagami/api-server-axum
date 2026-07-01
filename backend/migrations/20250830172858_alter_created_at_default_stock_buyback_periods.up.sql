-- Add up migration script here

-- 修正 created_at 預設值為函數 now()
ALTER TABLE stock_buyback_periods
ALTER COLUMN created_at SET DEFAULT now();
