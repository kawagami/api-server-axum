CREATE TABLE stock_ex_rights (
    stock_no     TEXT NOT NULL,
    ex_date      DATE NOT NULL,
    close_before DOUBLE PRECISION NOT NULL DEFAULT 0,
    cash_div     DOUBLE PRECISION NOT NULL DEFAULT 0,
    stock_rate   DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (stock_no, ex_date)
);
