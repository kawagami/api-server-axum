-- logs 查詢/清理都依 created_at,補索引避免全表掃(清理 job 見 jobs/cleanup_logs.rs)
CREATE INDEX IF NOT EXISTS idx_logs_created_at ON public.logs (created_at);
