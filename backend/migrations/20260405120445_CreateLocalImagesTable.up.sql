-- Add up migration script here

CREATE TABLE IF NOT EXISTS images (
    id serial PRIMARY KEY,
    storage_key text NOT NULL UNIQUE, -- 對應 storage 的唯一識別 key，方便刪除或搬移
    url text NOT NULL,                -- 公開存取的 URL
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
