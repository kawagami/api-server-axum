-- 權限全面收攏給 super_admin：清空 role_permissions，讓 guest / member / admin 皆為零權限。
-- super_admin 的全權來自程式 (repositories/roles.rs get_user_permission_strings_by_email
-- 偵測 user_roles 掛有 super_admin 角色 → 直接回傳 permissions 整張表)，不看此表，故不受影響。
-- 之後各角色權限一律由 super_admin 於後台「角色」頁 (PUT /admin/roles/{id}/permissions) 設定。
DELETE FROM role_permissions;

-- 新增管理員時預設勾選的角色（逗號分隔角色名稱），super_admin 於後台 Settings 調整
INSERT INTO app_settings (key, value, description, category)
VALUES ('new_user_default_roles', 'admin', '新增管理員時預設勾選的角色（逗號分隔角色名稱）', 'user')
ON CONFLICT (key) DO NOTHING;
