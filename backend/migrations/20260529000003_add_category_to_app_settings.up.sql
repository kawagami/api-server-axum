ALTER TABLE app_settings ADD COLUMN category TEXT NOT NULL DEFAULT 'general';

UPDATE app_settings SET category = 'integration' WHERE key = 'hackmd_token';
UPDATE app_settings SET category = 'oauth'       WHERE key IN ('google_client_id', 'google_redirect_url', 'github_client_id', 'github_redirect_url', 'line_client_id', 'line_redirect_url');
UPDATE app_settings SET category = 'storage'     WHERE key = 'upload_base_url';
UPDATE app_settings SET category = 'cors'        WHERE key = 'cors_allowed_origins';
