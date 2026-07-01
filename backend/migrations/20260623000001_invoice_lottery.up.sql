-- 發票登錄（對獎唯一真實來源，與記帳解耦）
CREATE TABLE invoices (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id       BIGINT NOT NULL,
    invoice_number  TEXT NOT NULL,          -- 完整字軌號碼，如 'AB12345678'
    invoice_date    DATE NOT NULL,
    period          TEXT NOT NULL,          -- 對獎期別 key 'YYYYMM'（期末偶數月）
    amount          NUMERIC(14,2),
    seller_tax_id   TEXT,
    source          TEXT NOT NULL,          -- 'qr' | 'barcode' | 'manual'
    ledger_entry_id UUID REFERENCES ledger_entries(id) ON DELETE SET NULL,
    lottery_checked BOOLEAN NOT NULL DEFAULT false,
    prize_tier      TEXT,                   -- 命中獎別；checked=true 且 null = 確定未中
    notified_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_invoices_member_number ON invoices (member_id, invoice_number);
CREATE INDEX idx_invoices_period_unchecked ON invoices (period) WHERE lottery_checked = false;

-- 每期中獎號碼
CREATE TABLE invoice_lottery_numbers (
    id          SERIAL PRIMARY KEY,
    period      TEXT NOT NULL,
    prize_tier  TEXT NOT NULL,             -- 'special' | 'grand' | 'first' | 'additional'
    number      TEXT NOT NULL,             -- special/grand/first = 8 碼；additional = 3 碼
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (period, prize_tier, number)
);

-- member 中獎 email 通知偏好（預設關閉，須主動開啟且有 email）
ALTER TABLE members ADD COLUMN lottery_notify_enabled BOOLEAN NOT NULL DEFAULT false;

-- 對獎已改由 invoices 表負責去重；ledger_entries.invoice_number 僅供顯示，不再唯一
DROP INDEX IF EXISTS idx_ledger_entries_member_invoice;

-- admin 手動補中獎號碼用的權限
INSERT INTO permissions (resource, action, description) VALUES
    ('invoice_lottery', 'write', '手動輸入統一發票中獎號碼')
ON CONFLICT (resource, action) DO NOTHING;
