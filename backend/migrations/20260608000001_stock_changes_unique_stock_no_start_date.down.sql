ALTER TABLE stock_changes
    DROP CONSTRAINT stock_changes_stock_no_start_date_key;

ALTER TABLE stock_changes
    ADD CONSTRAINT stock_changes_stock_no_start_date_end_date_key UNIQUE (stock_no, start_date, end_date);
