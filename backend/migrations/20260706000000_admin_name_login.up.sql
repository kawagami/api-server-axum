-- 管理員登入識別改用 name（唯一），email 降為選填
-- 前提：現有 users.name 需已唯一（若有重複此步會失敗，須先處理）
ALTER TABLE users DROP CONSTRAINT users_email_unique;
ALTER TABLE users ALTER COLUMN email DROP NOT NULL;
ALTER TABLE users ADD CONSTRAINT users_name_unique UNIQUE (name);
