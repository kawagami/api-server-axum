-- 圖片壓縮相關設定（可於 /admin/settings 熱調整）
-- image_webp_quality 只有後端讀；三個 image_client_* 走 PUBLIC_KEYS 下發給前端上傳前壓縮。
INSERT INTO public.app_settings (key, value, description, category) VALUES
    ('image_webp_quality', '80', '後端 WebP 重編碼品質（1–100）', 'storage'),
    ('image_client_compress', 'true', '前端上傳前壓縮開關（true/false）', 'storage'),
    ('image_client_quality', '80', '前端壓縮品質（1–100）', 'storage'),
    ('image_client_max_edge', '2560', '前端壓縮長邊上限（px）', 'storage')
ON CONFLICT (key) DO NOTHING;
