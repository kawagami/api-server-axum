CREATE TABLE stock_ex_rights_checked (
    stock_no   TEXT NOT NULL,
    from_date  DATE NOT NULL,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (stock_no, from_date)
);
