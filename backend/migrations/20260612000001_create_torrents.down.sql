DROP TABLE IF EXISTS torrents;
DELETE FROM permissions WHERE resource = 'torrent';
DELETE FROM app_settings WHERE key IN ('torrent_max_active', 'torrent_retention_days', 'torrent_max_total_size_gb');
