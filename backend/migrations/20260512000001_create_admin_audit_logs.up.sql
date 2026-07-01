CREATE TABLE admin_audit_logs (
    id BIGSERIAL PRIMARY KEY,
    user_email VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    path TEXT NOT NULL,
    query TEXT,
    status_code SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_admin_audit_logs_created_at ON admin_audit_logs (created_at DESC);
CREATE INDEX idx_admin_audit_logs_user_email ON admin_audit_logs (user_email);
