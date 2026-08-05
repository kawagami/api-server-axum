-- admin_audit_logs 補 request_id：原本 audit 列與 logs 表完全對不起來。
-- audit 寫入曾經是「每請求 spawn 一個 task 直接 INSERT」，spawn 之後脫離 request span，
-- 拿不到 request_id；改走批次 channel 後可以在 middleware 內就取到值一起送。
-- 有了它，「誰在什麼時候打了哪支端點」與「那次請求的 WARN/ERROR 軌跡」可以互查。
ALTER TABLE admin_audit_logs ADD COLUMN request_id text;

-- partial index：舊資料與非請求路徑都是 NULL，不必進索引（與 logs.request_id 同策略）
CREATE INDEX IF NOT EXISTS admin_audit_logs_request_id_idx
    ON admin_audit_logs (request_id)
    WHERE request_id IS NOT NULL;
