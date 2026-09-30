ALTER TABLE stock_ex_rights_checked DROP COLUMN covered_until;

ALTER TABLE stock_ex_rights
    DROP COLUMN ref_price,
    ADD COLUMN cash_div DOUBLE PRECISION DEFAULT 0 NOT NULL,
    ADD COLUMN stock_rate DOUBLE PRECISION DEFAULT 0 NOT NULL;
