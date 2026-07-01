CREATE TABLE ledger_entries (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id   BIGINT NOT NULL,
    kind        TEXT NOT NULL,              -- 'income' | 'expense'
    amount      NUMERIC(14,2) NOT NULL,     -- 金額：用 NUMERIC 避免浮點誤差
    category    TEXT NOT NULL,              -- 全站固定分類（見 structs/ledger.rs）
    note        TEXT,
    occurred_at DATE NOT NULL,              -- 記帳日期
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ledger_entries_member_date ON ledger_entries (member_id, occurred_at DESC);
