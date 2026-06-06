INSERT INTO app_settings (key, value, description, category) VALUES
    ('smtp_username',  '', 'Gmail 寄件帳號',          'notification'),
    ('smtp_password',  '', 'Gmail App Password',       'notification'),
    ('notify_email',   '', '通知收件信箱（空白 = 同寄件帳號）', 'notification')
ON CONFLICT (key) DO NOTHING;
