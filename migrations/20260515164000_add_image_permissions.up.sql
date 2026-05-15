INSERT INTO permissions (resource, action, description) VALUES
    ('image', 'read',   '查看圖片列表'),
    ('image', 'write',  '上傳圖片'),
    ('image', 'delete', '刪除圖片')
ON CONFLICT (resource, action) DO NOTHING;
