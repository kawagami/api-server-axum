-- 刪除冗余 index（前綴已被其他 composite/unique index 覆蓋）
DROP INDEX IF EXISTS idx_stock_day_all_stock_code;        -- 被 (stock_code, trade_date DESC) 覆蓋
DROP INDEX IF EXISTS idx_stock_day_all_trade_date;        -- 被 UNIQUE (trade_date, stock_code) 覆蓋
DROP INDEX IF EXISTS idx_stock_buyback_periods_stock_no;  -- 被 UNIQUE (stock_no, start_date) 覆蓋
