-- 訪客留言給站長:公開端 POST 提交(不需登入),後台閱讀/刪除。
-- name / email 皆選填(可匿名留言;留 email 站長才回得了信),content 必填。
CREATE TABLE public.messages (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name text,
    email text,
    content text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_messages_created_at ON public.messages (created_at DESC);

-- 後台閱讀/刪除留言權限(super_admin 自動取全部,這裡另授予 admin 角色)
INSERT INTO permissions (resource, action, description)
VALUES ('message', 'read', '查看訪客留言'),
       ('message', 'delete', '刪除訪客留言');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin' AND p.resource = 'message' AND p.action IN ('read', 'delete');
