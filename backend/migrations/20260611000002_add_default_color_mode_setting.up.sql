INSERT INTO app_settings (key, value, description, category) VALUES
    ('default_color_mode', 'system', '全站深淺色預設（light / dark / system）', 'appearance')
ON CONFLICT (key) DO NOTHING;
