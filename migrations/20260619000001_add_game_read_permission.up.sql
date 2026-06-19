INSERT INTO permissions (resource, action, description) VALUES
    ('game', 'read', '查詢即時對局總覽')
ON CONFLICT (resource, action) DO NOTHING;
