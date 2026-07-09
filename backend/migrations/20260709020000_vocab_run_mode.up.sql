-- 對局模式(生存 / 限時 / 限時生存),讓最佳紀錄能分模式比較
ALTER TABLE vocab_runs ADD COLUMN mode TEXT NOT NULL DEFAULT 'survival';

CREATE INDEX idx_vocab_runs_member_mode_best
    ON vocab_runs (member_id, mode, correct_count DESC, max_combo DESC);
