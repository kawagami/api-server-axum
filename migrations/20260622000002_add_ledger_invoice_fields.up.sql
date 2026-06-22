ALTER TABLE ledger_entries
    ADD COLUMN invoice_number TEXT,
    ADD COLUMN seller_tax_id  TEXT,
    ADD COLUMN source         TEXT NOT NULL DEFAULT 'manual';  -- 'manual' | 'invoice_qr'

-- 同一 member 的同一張發票只能匯入一次（手動建立 invoice_number 為 NULL，不受限）
CREATE UNIQUE INDEX idx_ledger_entries_member_invoice
    ON ledger_entries (member_id, invoice_number)
    WHERE invoice_number IS NOT NULL;
