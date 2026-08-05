DROP INDEX IF EXISTS admin_audit_logs_request_id_idx;
ALTER TABLE admin_audit_logs DROP COLUMN IF EXISTS request_id;
