DROP INDEX IF EXISTS idx_blogs_markdown_trgm;
-- 刻意不 DROP EXTENSION pg_trgm：擴充是資料庫層級的共用物件，未來其他索引可能已依賴它，
-- 而它本身不佔成本。要清乾淨請手動確認沒有其他 *_trgm_ops 索引後再 drop。
