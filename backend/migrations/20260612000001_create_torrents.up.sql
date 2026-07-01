CREATE TABLE IF NOT EXISTS torrents (
    id           SERIAL PRIMARY KEY,
    info_hash    TEXT NOT NULL UNIQUE,
    magnet_uri   TEXT NOT NULL,
    name         TEXT,
    status       TEXT NOT NULL DEFAULT 'pending',
    total_size   BIGINT,
    files        JSONB,
    error        TEXT,
    created_by   TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_torrents_status ON torrents (status);

INSERT INTO permissions (resource, action, description) VALUES
    ('torrent', 'read', '查詢 torrent 任務與下載檔案'),
    ('torrent', 'create', '新增 torrent 任務'),
    ('torrent', 'delete', '刪除 torrent 任務與檔案')
ON CONFLICT (resource, action) DO NOTHING;

INSERT INTO app_settings (key, value, description, category) VALUES
    ('torrent_max_active', '2', '同時下載的 torrent 數量上限', 'torrent'),
    ('torrent_retention_days', '7', 'completed / failed 後保留天數，逾期自動清除', 'torrent'),
    ('torrent_max_total_size_gb', '20', 'torrent 目錄總容量上限（GB），超過拒收新任務', 'torrent')
ON CONFLICT (key) DO NOTHING;
