-- 模糊預覽（blur-up）：上傳時順手存原圖寬高與一張極小 WebP 的 data URL，
-- 讓前端 next/image 在原圖載入前先顯示模糊版，並用真實寬高預留版位（免 CLS）。
-- 皆可為 NULL：既有圖片由啟動時的 backfill_placeholders 補；decode 失敗的列維持 NULL，前端退回無 placeholder。
ALTER TABLE images
    ADD COLUMN width INTEGER,
    ADD COLUMN height INTEGER,
    ADD COLUMN blur_data_url TEXT;
