CREATE TABLE role_permissions (
    role_id       INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- guest: 公開讀取
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'guest' AND (
    (p.resource = 'blog'  AND p.action = 'read') OR
    (p.resource = 'image' AND p.action = 'read') OR
    (p.resource = 'note'  AND p.action = 'read')
);

-- member: guest + 新增部分內容
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'member' AND (
    (p.resource = 'blog'  AND p.action IN ('read', 'create')) OR
    (p.resource = 'image' AND p.action IN ('read', 'create')) OR
    (p.resource = 'note'  AND p.action = 'read')
);

-- admin: 全部非 role 相關
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'admin' AND p.resource != 'role';

-- super_admin: 全部
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id FROM roles r, permissions p
WHERE r.name = 'super_admin';
