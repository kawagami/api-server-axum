-- WebAuthn user handle：spec 建議用不含 PII 的隨機值，不用 bigint id 硬轉（洩漏 id 且非隨機）
ALTER TABLE users ADD COLUMN webauthn_user_handle uuid NOT NULL UNIQUE DEFAULT gen_random_uuid();

-- 一個 user 可有多把 passkey；passkey 欄整包序列化 webauthn_rs::prelude::Passkey（公鑰/counter/backup 旗標）
CREATE TABLE user_passkeys (
    id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    credential_id text NOT NULL UNIQUE,
    passkey jsonb NOT NULL,
    label varchar(64) NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);

CREATE INDEX idx_user_passkeys_user_id ON user_passkeys (user_id);
