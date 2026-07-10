-- 還原 HackMD 資料表結構與設定(資料不可還原)
CREATE TABLE public.hackmd_posts (
    id text NOT NULL,
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
    user_path text NOT NULL,
    write_permission text NOT NULL
);

ALTER TABLE ONLY public.hackmd_posts
    ADD CONSTRAINT hackmd_posts_pkey PRIMARY KEY (id);

CREATE TABLE public.hackmd_users (
    user_path text NOT NULL,
    biography text,
    name text NOT NULL,
    photo text NOT NULL
);

ALTER TABLE ONLY public.hackmd_users
    ADD CONSTRAINT hackmd_users_pkey PRIMARY KEY (user_path);

INSERT INTO public.app_settings (key, value, description, category) VALUES ('hackmd_token', '', 'HackMD API Token', 'integration');
