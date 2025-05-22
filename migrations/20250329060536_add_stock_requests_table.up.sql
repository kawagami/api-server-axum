-- Add up migration script here

CREATE TABLE stock_changes (
    id SERIAL PRIMARY KEY,
    stock_no TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    stock_name TEXT,
    start_price DOUBLE PRECISION,
    end_price DOUBLE PRECISION,
    change DOUBLE PRECISION,
	created_at TIMESTAMPTZ NULL DEFAULT 'now()',
	updated_at TIMESTAMPTZ NULL DEFAULT 'now()',
    UNIQUE (stock_no, start_date, end_date)  -- 唯一約束
);

-- 創建索引
CREATE INDEX idx_stock_changes_status ON stock_changes (status);
