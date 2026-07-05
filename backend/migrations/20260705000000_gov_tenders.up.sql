-- 政府電子採購網標案追蹤
CREATE TABLE gov_tenders (
    id BIGSERIAL PRIMARY KEY,
    filename TEXT NOT NULL UNIQUE,                 -- 公告檔名（來源 API 唯一鍵）
    date DATE NOT NULL,                            -- 公告日期
    tender_type TEXT NOT NULL,                     -- 公告類型（招標 / 決標 / 無法決標…）
    title TEXT NOT NULL,                           -- 標案名稱
    category TEXT,                                 -- 標的分類
    unit_id TEXT NOT NULL,
    unit_name TEXT NOT NULL,                       -- 機關名稱
    job_number TEXT NOT NULL,                      -- 標案案號
    companies JSONB NOT NULL DEFAULT '[]'::jsonb,  -- 廠商名稱陣列
    keyword TEXT NOT NULL,                         -- 由哪個追蹤關鍵字撈到
    detail_url TEXT NOT NULL,                      -- 官方公告頁連結
    notified_at TIMESTAMPTZ,                       -- 已寄出新標案通知的時間
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_gov_tenders_date ON gov_tenders (date DESC, id DESC);
CREATE INDEX idx_gov_tenders_tender_type ON gov_tenders (tender_type);

INSERT INTO app_settings (key, value, description, category)
VALUES ('gov_tender_keywords', '網站', '政府採購網標案追蹤關鍵字（逗號分隔，留空 = 停用抓取）', 'gov_tender');

INSERT INTO permissions (resource, action, description)
VALUES ('gov_tender', 'read', '查詢政府採購網標案');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin' AND p.resource = 'gov_tender' AND p.action = 'read';
