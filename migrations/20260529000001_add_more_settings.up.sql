INSERT INTO app_settings (key, value, description) VALUES
    ('cors_allowed_origins', 'https://kawa.homes',        'CORS 允許來源（逗號分隔多個）'),
    ('upload_base_url',      'https://axum.kawa.homes/uploads','Upload Base URL（重啟後生效）'),
    ('google_client_id',     '',                          'Google OAuth Client ID'),
    ('google_redirect_url',  '',                          'Google OAuth Redirect URL'),
    ('github_client_id',     '',                          'GitHub OAuth Client ID'),
    ('github_redirect_url',  '',                          'GitHub OAuth Redirect URL'),
    ('line_client_id',       '',                          'LINE OAuth Client ID'),
    ('line_redirect_url',    '',                          'LINE OAuth Redirect URL')
ON CONFLICT (key) DO NOTHING;
