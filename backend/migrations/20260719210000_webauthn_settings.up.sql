-- WebAuthn RP 設定改走 app_settings（平台保留 key，platform:read/update 才碰得到），不吃 env。
-- 預設值對齊本地開發；生產/商家 instance 部署後到 /admin/platform 填自己的網域（一次性）。
-- ⚠️ rp_id 一旦有使用者建立 passkey 就不可再改（改 = 既有 passkey 全數作廢）。
INSERT INTO app_settings (key, value, description, category)
VALUES
    (
        'webauthn_rp_id',
        'localhost',
        'Passkey RP ID：綁定的網域錨點（如 kawa.homes，涵蓋所有子網域）。有 passkey 後不可改，改了全數作廢',
        'platform'
    ),
    (
        'webauthn_rp_origin',
        'http://localhost:3000',
        'Passkey origin：登入頁實際所在的瀏覽器來源（前端網域，如 https://kawa.homes）',
        'platform'
    )
ON CONFLICT (key) DO NOTHING;
