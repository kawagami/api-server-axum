-- instance 級功能開關：'all' = 全開；JSON 字串陣列 = 明確白名單（商家 instance 用）
INSERT INTO app_settings (key, value, description, category)
VALUES (
    'enabled_features',
    'all',
    '啟用的功能（all = 全開，或 JSON 字串陣列如 ["blog","tools"]；控制 API 路由與排程任務）',
    'features'
)
ON CONFLICT (key) DO NOTHING;
