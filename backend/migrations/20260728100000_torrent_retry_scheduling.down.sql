DELETE FROM app_settings WHERE key = 'torrent_metadata_timeout_seconds';

DROP INDEX IF EXISTS torrents_resumable_idx;

ALTER TABLE torrents
    DROP COLUMN IF EXISTS attempt_count,
    DROP COLUMN IF EXISTS last_attempt_at;
