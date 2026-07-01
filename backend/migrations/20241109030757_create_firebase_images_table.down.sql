-- Add down migration script here
-- 刪除觸發器
DROP TRIGGER IF EXISTS update_firebase_images_timestamp ON firebase_images;

-- 刪除觸發器所用的函數
DROP FUNCTION IF EXISTS update_timestamp;

-- 刪除 firebase_images 表
DROP TABLE IF EXISTS firebase_images;

