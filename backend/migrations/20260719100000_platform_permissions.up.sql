-- 平台保留設定的專屬權限（頁面 /admin/platform）：
-- 商家管理員拿 setting:read/update 管日常設定，保留 key（如 enabled_features）只有 platform:* 能看/改。
-- super_admin 自動擁有全部 permissions，不需指派。
INSERT INTO permissions (resource, action, description)
VALUES ('platform', 'read', '查看平台保留設定(功能開關等)'),
       ('platform', 'update', '修改平台保留設定(功能開關等)');
