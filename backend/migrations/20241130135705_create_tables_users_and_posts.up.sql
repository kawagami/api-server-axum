-- Add up migration script here
-- 使用者表 (users)
CREATE TABLE hackmd_users(
    user_path text PRIMARY KEY,
    biography text,
    name text NOT NULL,
    photo text NOT NULL
);

-- 文章表 (posts)
CREATE TABLE hackmd_posts(
    id text PRIMARY KEY,
    content text,
    created_at bigint NOT NULL,
    last_changed_at bigint NOT NULL,
    permalink text,
    publish_link text,
    publish_type text NOT NULL,
    published_at bigint,
    read_permission text NOT NULL,
    short_id text NOT NULL,
    tags text[],
    tags_updated_at bigint,
    team_path text,
    title text NOT NULL,
    title_updated_at bigint NOT NULL,
    user_path text NOT NULL, -- last_change_user key
    write_permission text NOT NULL
);

-- 為 tags 欄位新增 GIN 索引
-- 測試後 cost 還比較高 先不放了
-- CREATE INDEX idx_hackmd_posts_tags ON hackmd_posts USING GIN(tags);

