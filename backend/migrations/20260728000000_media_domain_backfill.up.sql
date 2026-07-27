-- 存量圖片 URL 從舊網域 axum.kawa.homes/uploads/ 回填成 media.kawa.homes/
--
-- 兩個網域指向同一個磁碟目錄（nginx `alias /srv/kawa/uploads/` 與 `root
-- /srv/kawa/uploads`），所以這是純字串替換：零檔案搬動、零下載、對 R2 遷移計畫無影響。
--
-- ⚠ 兩個 UPDATE 必須在同一個 transaction（sqlx 每個 migration 檔自成一個 tx）。
-- services/blogs.rs 的 active/orphaned 標記是拿 blogs.markdown 抽出的 URL 與
-- images.url 做「精確字串比對」，只改一邊會讓下次編輯該文章時把舊 URL 標回
-- unused、新 URL 又對不到任何一列 → cleanup_unused_images 一小時後連檔案一起刪。
--
-- 刻意不動 updated_at：這是 URL 搬遷，不是內容更新，不該讓全站文章的「最後更新」
-- 時間一起跳到今天（前端有依此排序/顯示）。

UPDATE images
   SET url = replace(url, 'https://axum.kawa.homes/uploads/', 'https://media.kawa.homes/')
 WHERE url LIKE 'https://axum.kawa.homes/uploads/%';

UPDATE blogs
   SET markdown = replace(markdown, 'https://axum.kawa.homes/uploads/', 'https://media.kawa.homes/')
 WHERE markdown LIKE '%https://axum.kawa.homes/uploads/%';

-- baseline 的 seed 值仍是舊網域（已上線的 migration 有 checksum，不能回頭改），
-- 本站生產環境早已由 admin 熱更新成 media，這行是為了讓全新 instance 開箱一致。
UPDATE app_settings
   SET value = 'https://media.kawa.homes'
 WHERE key = 'upload_base_url'
   AND value = 'https://axum.kawa.homes/uploads';
