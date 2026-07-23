-- blog 留言:訪客 + 會員混合。公開端 GET 列表(不需登入)、POST 提交(帶 member token 綁 member_id,否則訪客)。
-- 後台閱讀/刪除。member_id 有值 = 會員留言(顯示名/頭像取 members 表);null = 訪客留言(顯示 author_name)。
-- author_name 為訪客自填名(選填,可匿名);會員留言不用此欄。content 必填。
CREATE TABLE public.blog_comments (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    blog_id uuid NOT NULL REFERENCES public.blogs (id) ON DELETE CASCADE,
    member_id bigint REFERENCES public.members (id) ON DELETE SET NULL,
    author_name text,
    content text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_blog_comments_blog_created ON public.blog_comments (blog_id, created_at DESC);

-- 後台閱讀/刪除留言權限(super_admin 自動取全部,這裡另授予 admin 角色)
INSERT INTO permissions (resource, action, description)
VALUES ('comment', 'read', '查看部落格留言'),
       ('comment', 'delete', '刪除部落格留言');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin' AND p.resource = 'comment' AND p.action IN ('read', 'delete');
