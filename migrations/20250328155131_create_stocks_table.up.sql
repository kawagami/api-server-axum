-- Add up migration script here

-- 額外紀錄股票代號對照的 table 目前價格不參考此表資料
CREATE TABLE stocks (
    id SERIAL PRIMARY KEY,
    code VARCHAR(10) NOT NULL UNIQUE,
    name TEXT NOT NULL,
    closing_price DECIMAL(10,2) NOT NULL,
    monthly_average_price DECIMAL(10,2) NOT NULL
);
