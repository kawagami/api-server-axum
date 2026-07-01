-- 每日不重複到訪人數（以台北日期為界）。即時計數走 Redis HyperLogLog，
-- 每日 job 將前一日 PFCOUNT 落地此表做長期趨勢。
CREATE TABLE daily_visitor_stats (
    date            DATE PRIMARY KEY,        -- 台北日期（UTC+8 日界）
    unique_visitors BIGINT NOT NULL,         -- 當日不重複到訪（HLL 近似值）
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO permissions (resource, action, description) VALUES
    ('stat', 'read', '查詢網站流量統計')
ON CONFLICT (resource, action) DO NOTHING;
