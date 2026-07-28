UPDATE app_settings
SET description = 'magnet metadata 解析逾時秒數：這段時間內找不到 peers 就放棄本輪、排到隊尾讓其他任務先試（累積 3 次才判失敗）'
WHERE key = 'torrent_metadata_timeout_seconds';
