-- 冷門種子不再獨占啟動名額。
-- magnet 的 metadata 解析（DHT 找 peers）沒 peers 就是乾等，逾時之前那筆會一直佔著
-- torrent_max_active 的一格，後面排隊的完全不會被嘗試。
-- 改成：每次啟動嘗試都記一筆時間，候選排序讓「最久沒試過的」優先（沒試過的最優先），
-- metadata 逾時累積到上限次數才判 failed，中間各輪只是排到隊尾等下一輪。
ALTER TABLE torrents
    ADD COLUMN last_attempt_at timestamp with time zone,
    ADD COLUMN attempt_count integer DEFAULT 0 NOT NULL;

-- list_resumable 的排序鍵（只有待啟動的兩種狀態會被掃）
CREATE INDEX torrents_resumable_idx ON torrents (last_attempt_at NULLS FIRST, id)
    WHERE status IN ('pending', 'downloading');

INSERT INTO app_settings (key, value, description, category)
VALUES
    (
        'torrent_metadata_timeout_seconds',
        '180',
        'magnet metadata 解析逾時秒數：這段時間內找不到 peers 就放棄本輪、排到隊尾讓其他任務先試（累積 3 次才判失敗）',
        'torrent'
    )
ON CONFLICT (key) DO NOTHING;
