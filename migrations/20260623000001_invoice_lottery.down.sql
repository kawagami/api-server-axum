DELETE FROM permissions WHERE resource = 'invoice_lottery' AND action = 'write';

-- 還原 ledger_entries 發票號碼唯一索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_ledger_entries_member_invoice
    ON ledger_entries (member_id, invoice_number)
    WHERE invoice_number IS NOT NULL;

ALTER TABLE members DROP COLUMN IF EXISTS lottery_notify_enabled;

DROP TABLE IF EXISTS invoice_lottery_numbers;
DROP TABLE IF EXISTS invoices;
