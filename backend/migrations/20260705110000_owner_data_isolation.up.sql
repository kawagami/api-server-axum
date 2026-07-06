-- 多 admin 資料隔離：blogs / images / torrents 記錄擁有者 user id（FK → users，刪 user 連帶清）
ALTER TABLE blogs ADD COLUMN author_id bigint;
ALTER TABLE images ADD COLUMN owner_id bigint;
ALTER TABLE torrents ADD COLUMN owner_id bigint;

ALTER TABLE blogs ADD CONSTRAINT blogs_author_id_fkey
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE images ADD CONSTRAINT images_owner_id_fkey
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE torrents ADD CONSTRAINT torrents_owner_id_fkey
    FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE;

-- torrents 既有 created_by（email）對回 users.id
UPDATE torrents t SET owner_id = u.id FROM users u WHERE t.created_by = u.email;

-- blogs / images 既有「無主」資料歸第一個 super_admin（站長既有內容）；無 super_admin 則維持 NULL
UPDATE blogs SET author_id = sa.id
FROM (SELECT ur.user_id AS id FROM user_roles ur JOIN roles r ON ur.role_id = r.id
      WHERE r.name = 'super_admin' ORDER BY ur.user_id LIMIT 1) sa
WHERE blogs.author_id IS NULL;

UPDATE images SET owner_id = sa.id
FROM (SELECT ur.user_id AS id FROM user_roles ur JOIN roles r ON ur.role_id = r.id
      WHERE r.name = 'super_admin' ORDER BY ur.user_id LIMIT 1) sa
WHERE images.owner_id IS NULL;

CREATE INDEX idx_blogs_author_id ON blogs(author_id);
CREATE INDEX idx_images_owner_id ON images(owner_id);
CREATE INDEX idx_torrents_owner_id ON torrents(owner_id);
