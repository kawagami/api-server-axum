-- 英文單字闖關:題庫 + 對局紀錄 + 學習進度 + 會員經驗值

-- 全站會員經驗值(等級由 exp 換算,不落欄位)
ALTER TABLE members ADD COLUMN exp BIGINT NOT NULL DEFAULT 0;

-- 題庫
CREATE TABLE words (
    id BIGSERIAL PRIMARY KEY,
    word TEXT NOT NULL UNIQUE,                    -- 全小寫正規化
    part_of_speech TEXT NOT NULL,                 -- n. / v. / adj. / adv. …
    meaning_zh TEXT NOT NULL,                     -- 繁中釋義
    example_sentence TEXT NOT NULL,               -- 例句(含該單字原形,填空題挖空用)
    difficulty SMALLINT NOT NULL CHECK (difficulty BETWEEN 1 AND 5),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,        -- 下架不出題,不刪資料
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_words_difficulty ON words (difficulty) WHERE enabled;

-- 每局生存模式結果(進行中狀態在 Redis,結束才落地)
CREATE TABLE vocab_runs (
    id UUID PRIMARY KEY,
    member_id BIGINT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    answered_count INT NOT NULL,
    correct_count INT NOT NULL,
    max_combo INT NOT NULL,
    exp_gained BIGINT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_vocab_runs_member_best ON vocab_runs (member_id, correct_count DESC, max_combo DESC);

-- 會員 × 單字學習進度(未來複習模式 / 熟練度的地基)
CREATE TABLE member_word_stats (
    member_id BIGINT NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    word_id BIGINT NOT NULL REFERENCES words(id) ON DELETE CASCADE,
    correct_count INT NOT NULL DEFAULT 0,
    wrong_count INT NOT NULL DEFAULT 0,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member_id, word_id)
);
