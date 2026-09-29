ALTER INDEX idx_admin_audit_logs_actor RENAME TO idx_admin_audit_logs_user_email;
ALTER TABLE admin_audit_logs RENAME COLUMN actor TO user_email;
