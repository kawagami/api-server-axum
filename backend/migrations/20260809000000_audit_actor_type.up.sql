-- 稽核從「只記 admin」擴到也記 member 的寫入操作。
--
-- `/member/*`（portfolio / ledger / invoices / lotto）走的是 `authorize_member`，
-- 不經 `with_auth`，所以 audit middleware 從來沒跑到 —— 會員改了什麼、刪了什麼
-- 一直沒有任何紀錄，出事只能靠 DB 現值猜。
--
-- 既有列全部是 admin，故 DEFAULT 'admin' 直接回填，不必另外 UPDATE。
-- 不建索引：只有兩種值，選擇性太低；要縮範圍請先用 created_at（已有索引）。
ALTER TABLE admin_audit_logs
    ADD COLUMN IF NOT EXISTS actor_type text NOT NULL DEFAULT 'admin';
