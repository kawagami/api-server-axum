ALTER TABLE stock_changes
    ALTER COLUMN start_date TYPE TEXT USING
        LPAD((EXTRACT(YEAR FROM start_date)::INT - 1911)::TEXT, 3, '0') || TO_CHAR(start_date, 'MMDD'),
    ALTER COLUMN end_date TYPE TEXT USING
        LPAD((EXTRACT(YEAR FROM end_date)::INT - 1911)::TEXT, 3, '0') || TO_CHAR(end_date, 'MMDD');
