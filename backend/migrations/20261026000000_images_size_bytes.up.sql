-- 圖片檔案大小（實際落地的 bytes，即後端轉檔/原檔保留後的結果），供 /admin/images 顯示。
-- 可為 NULL：既有圖片由啟動時的 backfill_sizes 讀檔案大小補；檔案已不存在的列維持 NULL。
ALTER TABLE images ADD COLUMN size_bytes BIGINT;
