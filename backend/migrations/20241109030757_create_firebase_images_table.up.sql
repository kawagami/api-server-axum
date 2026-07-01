-- Add up migration script here
CREATE TABLE firebase_images (
    id SERIAL PRIMARY KEY,
    image_url TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 為 `updated_at` 欄位設定觸發器，使其在更新記錄時自動更新
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_firebase_images_timestamp
BEFORE UPDATE ON firebase_images
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
