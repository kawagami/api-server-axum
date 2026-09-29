-- admin_audit_logs.user_email 名實不符：admin 存的是顯示名（users.name，登入識別早已不是 email），
-- member 存的是 `member#{id}`。改名成 actor，與既有的 actor_type 成對。
-- 索引一併改名，免得之後有人照索引名去找一個不存在的欄位。
ALTER TABLE admin_audit_logs RENAME COLUMN user_email TO actor;
ALTER INDEX idx_admin_audit_logs_user_email RENAME TO idx_admin_audit_logs_actor;
