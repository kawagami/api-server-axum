ALTER TABLE stock_changes
    ALTER COLUMN start_date TYPE DATE USING TO_DATE(
        (CAST((CAST(start_date AS INT) + 19110000) AS TEXT)),
        'YYYYMMDD'
    ),
    ALTER COLUMN end_date TYPE DATE USING TO_DATE(
        (CAST((CAST(end_date AS INT) + 19110000) AS TEXT)),
        'YYYYMMDD'
    );
