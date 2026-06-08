-- Remove stale duplicates: keep the record whose end_date matches current stock_buyback_periods
DELETE FROM stock_changes
WHERE id IN (
    SELECT sc.id
    FROM stock_changes sc
    WHERE EXISTS (
        SELECT 1
        FROM stock_changes sc2
        WHERE sc2.stock_no = sc.stock_no
          AND sc2.start_date = sc.start_date
          AND sc2.id != sc.id
    )
    AND NOT EXISTS (
        SELECT 1
        FROM stock_buyback_periods bp
        WHERE bp.stock_no = sc.stock_no
          AND bp.start_date = sc.start_date
          AND bp.end_date = sc.end_date
    )
);

ALTER TABLE stock_changes
    DROP CONSTRAINT stock_changes_stock_no_start_date_end_date_key;

ALTER TABLE stock_changes
    ADD CONSTRAINT stock_changes_stock_no_start_date_key UNIQUE (stock_no, start_date);
