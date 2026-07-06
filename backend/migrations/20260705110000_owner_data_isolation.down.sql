DROP INDEX IF EXISTS idx_torrents_owner_id;
DROP INDEX IF EXISTS idx_images_owner_id;
DROP INDEX IF EXISTS idx_blogs_author_id;

ALTER TABLE torrents DROP CONSTRAINT IF EXISTS torrents_owner_id_fkey;
ALTER TABLE images DROP CONSTRAINT IF EXISTS images_owner_id_fkey;
ALTER TABLE blogs DROP CONSTRAINT IF EXISTS blogs_author_id_fkey;

ALTER TABLE torrents DROP COLUMN IF EXISTS owner_id;
ALTER TABLE images DROP COLUMN IF EXISTS owner_id;
ALTER TABLE blogs DROP COLUMN IF EXISTS author_id;
