-- 首頁功能卡片顯示與排序（JSON 字串陣列，前端依 registry 過濾未知 key）
INSERT INTO app_settings (key, value, description, category)
VALUES (
    'home_features',
    '["blog","vocab","games","ledger","portfolio","invoices","lotto","tools","about"]',
    '首頁功能卡片（JSON 字串陣列 = 顯示與排序，空陣列 = 全部隱藏）',
    'homepage'
);
