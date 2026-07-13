DELETE FROM role_permissions rp USING permissions p
WHERE rp.permission_id = p.id AND p.resource = 'metric';
DELETE FROM permissions WHERE resource = 'metric';
DROP TABLE IF EXISTS public.system_metrics;
