UPDATE app_settings
SET description = '網站風格主題（forest / ocean / sky / sunset / sakura / grape / mono / auto）'
WHERE key = 'site_theme';

INSERT INTO app_settings (key, value, description, category) VALUES
    ('theme_rotation',
     '{"0":"forest","1":"ocean","2":"sky","3":"sunset","4":"sakura","5":"grape","6":"mono"}',
     '每日輪播主題對應表（星期 0=週日..6=週六 → 主題），site_theme=auto 時生效',
     'appearance')
ON CONFLICT (key) DO NOTHING;
