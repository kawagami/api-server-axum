CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT,
    UNIQUE(resource, action)
);

INSERT INTO permissions (resource, action, description) VALUES
    ('blog',  'read',   '讀取文章'),
    ('blog',  'create', '新增文章'),
    ('blog',  'update', '編輯文章'),
    ('blog',  'delete', '刪除文章'),
    ('image', 'read',   '讀取圖片'),
    ('image', 'create', '上傳圖片'),
    ('image', 'delete', '刪除圖片'),
    ('note',  'read',   '讀取筆記'),
    ('note',  'create', '新增筆記'),
    ('note',  'update', '編輯筆記'),
    ('note',  'delete', '刪除筆記'),
    ('stock', 'read',   '讀取股票資料'),
    ('stock', 'create', '新增股票資料'),
    ('stock', 'update', '更新股票資料'),
    ('stock', 'delete', '刪除股票資料'),
    ('stock', 'manage', '管理股票任務'),
    ('user',  'read',   '讀取使用者'),
    ('user',  'create', '新增使用者'),
    ('user',  'update', '更新使用者'),
    ('user',  'delete', '刪除使用者'),
    ('role',  'read',   '讀取角色'),
    ('role',  'create', '新增角色'),
    ('role',  'update', '更新角色'),
    ('role',  'delete', '刪除角色'),
    ('role',  'assign', '指派角色給使用者');
