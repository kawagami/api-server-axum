INSERT INTO permissions (resource, action, description) VALUES
    ('stock', 'read',  '查看股票資料'),
    ('stock', 'write', '修改股票資料')
ON CONFLICT (resource, action) DO NOTHING;
