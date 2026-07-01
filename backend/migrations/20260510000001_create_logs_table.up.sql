CREATE TABLE logs (
    id BIGSERIAL PRIMARY KEY,
    level VARCHAR(10) NOT NULL,
    message TEXT NOT NULL,
    target VARCHAR(255) NOT NULL,
    file VARCHAR(255),
    line INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_logs_created_at ON logs (created_at DESC);
CREATE INDEX idx_logs_level ON logs (level);
