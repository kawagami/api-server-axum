INSERT INTO app_settings (key, value, description, category) VALUES
    ('site_theme', 'forest', '網站風格主題（forest / ocean）', 'appearance')
ON CONFLICT (key) DO NOTHING;
