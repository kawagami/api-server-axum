CREATE TABLE IF NOT EXISTS users (
    id                BIGSERIAL PRIMARY KEY,
    name              VARCHAR(255)                 NOT NULL,
    email             VARCHAR(255)                 NOT NULL,
    email_verified_at TIMESTAMP(0) WITHOUT TIME ZONE,
    password          VARCHAR(255)                 NOT NULL,
    remember_token    VARCHAR(100),
    created_at        TIMESTAMP(0) WITHOUT TIME ZONE,
    updated_at        TIMESTAMP(0) WITHOUT TIME ZONE,
    CONSTRAINT users_email_unique UNIQUE (email)
);
