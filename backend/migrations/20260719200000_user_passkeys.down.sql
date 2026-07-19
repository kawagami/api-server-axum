DROP TABLE IF EXISTS user_passkeys;
ALTER TABLE users DROP COLUMN IF EXISTS webauthn_user_handle;
