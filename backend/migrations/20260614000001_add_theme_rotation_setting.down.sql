DELETE FROM app_settings WHERE key = 'theme_rotation';

UPDATE app_settings
SET description = '網站風格主題（forest / ocean）'
WHERE key = 'site_theme';
