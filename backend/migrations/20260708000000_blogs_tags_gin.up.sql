-- 讓 tag 過濾（tags @> ARRAY[...]）與成員查詢走索引，取代對 blogs 的循序掃描
CREATE INDEX IF NOT EXISTS idx_blogs_tags ON blogs USING GIN (tags);
