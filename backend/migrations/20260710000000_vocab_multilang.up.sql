-- 單字闖關多語言化(日文版地基):words 加語言/讀音、vocab_runs 分語言、經驗分語言

-- 題庫多語言
ALTER TABLE words
    ADD COLUMN language TEXT NOT NULL DEFAULT 'en',
    ADD COLUMN reading TEXT,              -- 顯示用主讀音(平假名);英文為 NULL
    ADD COLUMN accepted_readings TEXT[];  -- 比對用全部合法讀音(平假名);NULL = 只接受 reading

-- 日文條目必有讀音(應用層也擋,DB 當最後防線)
ALTER TABLE words ADD CONSTRAINT words_ja_reading_required
    CHECK (language <> 'ja' OR reading IS NOT NULL);

-- 同表記多詞條(辛い=からい/つらい)→ 複合 unique;英文 reading 為 NULL 靠 NULLS NOT DISTINCT
-- 舊 UNIQUE(word) 是行內宣告、名稱由 PG 自動產生,動態查名再卸,不賭命名慣例
DO $$
DECLARE con_name text;
BEGIN
    SELECT conname INTO con_name FROM pg_constraint
    WHERE conrelid = 'words'::regclass AND contype = 'u';
    IF con_name IS NOT NULL THEN
        EXECUTE format('ALTER TABLE words DROP CONSTRAINT %I', con_name);
    END IF;
END $$;
ALTER TABLE words ADD CONSTRAINT words_lang_word_reading_key
    UNIQUE NULLS NOT DISTINCT (language, word, reading);

DROP INDEX idx_words_difficulty;
CREATE INDEX idx_words_lang_difficulty ON words (language, difficulty) WHERE enabled;

-- 對局分語言(個人最佳 per language per mode)
ALTER TABLE vocab_runs ADD COLUMN language TEXT NOT NULL DEFAULT 'en';
DROP INDEX idx_vocab_runs_member_best;
DROP INDEX idx_vocab_runs_member_mode_best;
CREATE INDEX idx_vocab_runs_member_lang_mode_best
    ON vocab_runs (member_id, language, mode, correct_count DESC, max_combo DESC);

-- 經驗分語言;members.exp 回填後凍結(vocab 讀寫全走本表,欄位留待日後清理)
CREATE TABLE member_vocab_exp (
    member_id BIGINT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    language  TEXT   NOT NULL,
    exp       BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (member_id, language)
);

INSERT INTO member_vocab_exp (member_id, language, exp)
SELECT id, 'en', exp FROM members WHERE exp > 0;
