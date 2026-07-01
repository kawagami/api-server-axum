DELETE FROM app_settings WHERE key IN (
    'cors_allowed_origins',
    'upload_base_url',
    'google_client_id',
    'google_redirect_url',
    'github_client_id',
    'github_redirect_url',
    'line_client_id',
    'line_redirect_url'
);
