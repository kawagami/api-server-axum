-- 單字題庫後台管理權限
INSERT INTO permissions (resource, action, description)
VALUES ('vocab', 'read', '查詢單字題庫'),
       ('vocab', 'update', '編輯單字題庫(釋義/難度/上下架)');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin' AND p.resource = 'vocab';
