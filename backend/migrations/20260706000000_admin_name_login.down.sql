-- 還原需 email 皆有值且唯一才可成功
ALTER TABLE users DROP CONSTRAINT users_name_unique;
ALTER TABLE users ALTER COLUMN email SET NOT NULL;
ALTER TABLE users ADD CONSTRAINT users_email_unique UNIQUE (email);
