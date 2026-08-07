-- logs 表的落地門檻，可熱更新（不重啟）。
-- 平時 WARN；線上要追一段沒有 WARN 的行為時暫時調到 INFO，查完調回。
-- 刻意沒有 DEBUG：那會把每個 4xx（含 bot 掃出來的 404）寫進 PG。
INSERT INTO app_settings (key, value, description, category)
VALUES (
    'log_db_level',
    'WARN',
    'logs 表落地門檻（ERROR / WARN / INFO）。調到 INFO 可暫時收 job 成敗、登入成功等事件，查完請調回 WARN；DEBUG 只能靠 RUST_LOG 走 stdout',
    'observability'
)
ON CONFLICT (key) DO NOTHING;
