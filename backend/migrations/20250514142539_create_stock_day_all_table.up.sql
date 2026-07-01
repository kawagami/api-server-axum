-- Add up migration script here

CREATE TABLE IF NOT EXISTS stock_day_all(
    id serial PRIMARY KEY,
    trade_date date NOT NULL, -- API 回傳的 date，格式是 yyyyMMdd，需要轉成 DATE 型態
    stock_code text NOT NULL, -- 證券代號
    stock_name text NOT NULL, -- 證券名稱
    trade_volume bigint, -- 成交股數（需移除千分號再轉為整數）
    trade_amount bigint, -- 成交金額
    open_price numeric(10, 2), -- 開盤價
    high_price numeric(10, 2), -- 最高價
    low_price numeric(10, 2), -- 最低價
    close_price numeric(10, 2), -- 收盤價
    price_change numeric(10, 2), -- 漲跌價差
    transaction_count integer, -- 成交筆數
    UNIQUE (trade_date, stock_code) -- 每天每檔股票只能有一筆紀錄
);
