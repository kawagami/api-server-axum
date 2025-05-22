-- Add down migration script here
-- 刪除索引
DROP INDEX IF EXISTS idx_hackmd_posts_tags;

-- 刪除表 (posts)
DROP TABLE IF EXISTS hackmd_posts;

-- 刪除表 (users)
DROP TABLE IF EXISTS hackmd_users;

