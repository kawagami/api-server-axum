DROP INDEX IF EXISTS idx_logs_request_id;
ALTER TABLE public.logs DROP COLUMN IF EXISTS fields;
ALTER TABLE public.logs DROP COLUMN IF EXISTS request_id;
