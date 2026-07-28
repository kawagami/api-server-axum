-- metadata 解析到點時不再無條件放棄：只有「有任務排不進 torrent_max_active 名額」時才讓位。
-- 沒人排隊就繼續等（不 abort，DHT 查詢進度留著），冷門種子可以慢慢磨。
-- 設定值語意因此從「逾時」變成「檢查是否讓位的間隔」，描述同步更新。
UPDATE app_settings
SET description = 'magnet metadata 解析的讓位檢查間隔（秒，預設 180）：每到點檢查一次，有任務排隊等名額才放棄本輪排到隊尾（累積 3 次判失敗）；沒人排隊就繼續等冷門種子'
WHERE key = 'torrent_metadata_timeout_seconds';
