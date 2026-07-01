-- Add up migration script here
CREATE TABLE IF NOT EXISTS stock_closing_prices (
    id SERIAL PRIMARY KEY,
    stock_no TEXT NOT NULL,
    date DATE NOT NULL,
    close_price DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_stock_closing_prices_stock_date ON stock_closing_prices (stock_no, date);
