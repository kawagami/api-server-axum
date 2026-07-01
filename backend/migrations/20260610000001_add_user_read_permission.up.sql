INSERT INTO permissions (resource, action, description) VALUES
    ('user', 'read', '查詢使用者列表')
ON CONFLICT (resource, action) DO NOTHING;
