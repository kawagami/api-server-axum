CREATE TABLE app_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT ''
);

INSERT INTO app_settings (key, value, description) VALUES
    ('hackmd_token', '', 'HackMD API Token');

INSERT INTO permissions (resource, action) VALUES
    ('setting', 'read'),
    ('setting', 'update')
ON CONFLICT DO NOTHING;
