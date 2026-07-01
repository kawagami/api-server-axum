DROP INDEX IF EXISTS idx_ledger_entries_member_invoice;

ALTER TABLE ledger_entries
    DROP COLUMN IF EXISTS invoice_number,
    DROP COLUMN IF EXISTS seller_tax_id,
    DROP COLUMN IF EXISTS source;
