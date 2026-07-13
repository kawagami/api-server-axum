-- VPS 系統指標歷史(每分鐘一筆,採集 job 見 jobs/collect_system_metrics.rs)
CREATE TABLE public.system_metrics (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    cpu_pct real NOT NULL,
    mem_used_mb integer NOT NULL,
    mem_total_mb integer NOT NULL,
    disk_used_mb integer NOT NULL,
    disk_total_mb integer NOT NULL,
    load1 real NOT NULL,
    load5 real NOT NULL,
    load15 real NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE INDEX idx_system_metrics_created_at ON public.system_metrics (created_at);

-- 觀測頁讀取權限(super_admin 動態取全部,這裡另授予 admin 角色)
INSERT INTO permissions (resource, action, description)
VALUES ('metric', 'read', '查詢系統指標(CPU/記憶體/磁碟/負載)');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin' AND p.resource = 'metric' AND p.action = 'read';
