DELETE FROM role_permissions rp USING permissions p
WHERE rp.permission_id = p.id AND p.resource = 'comment';
DELETE FROM permissions WHERE resource = 'comment';
DROP TABLE IF EXISTS public.blog_comments;
