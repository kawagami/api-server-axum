INSERT INTO permissions (resource, action, description) VALUES
    ('user', 'create', '新增使用者'),
    ('user', 'delete', '刪除使用者')
ON CONFLICT (resource, action) DO NOTHING;
