INSERT INTO app_settings (key, value, description, category) VALUES
    ('torrent_link_ttl_minutes', '180', '下載連結效期（分鐘）— 要涵蓋最大檔案在最慢線路的下載時間', 'torrent')
ON CONFLICT (key) DO NOTHING;
