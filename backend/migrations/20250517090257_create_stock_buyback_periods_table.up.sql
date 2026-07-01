-- Add up migration script here
CREATE TABLE IF NOT EXISTS stock_buyback_periods(
    id serial PRIMARY KEY,
    stock_no varchar(10) NOT NULL,
    start_date date NOT NULL, -- 儲存轉換後的西元日期
    end_date date NOT NULL,
    created_at TIMESTAMPTZ NULL DEFAULT 'now()',
    UNIQUE (stock_no, start_date, end_date)
);

-- 表註解
COMMENT ON TABLE stock_buyback_periods IS '執行庫藏股的股票代號與期間資訊';

-- 欄位註解
COMMENT ON COLUMN stock_buyback_periods.id IS '主鍵';
COMMENT ON COLUMN stock_buyback_periods.stock_no IS '股票代號';
COMMENT ON COLUMN stock_buyback_periods.start_date IS '庫藏股起始日期（西元）';
COMMENT ON COLUMN stock_buyback_periods.end_date IS '庫藏股結束日期（西元）';
COMMENT ON COLUMN stock_buyback_periods.created_at IS '建立時間';
