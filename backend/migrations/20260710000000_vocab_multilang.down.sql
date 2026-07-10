-- 經驗寫回 members.exp(各語言加總)
UPDATE members m SET exp = COALESCE(
    (SELECT SUM(e.exp) FROM member_vocab_exp e WHERE e.member_id = m.id), 0);
DROP TABLE member_vocab_exp;

DROP INDEX idx_vocab_runs_member_lang_mode_best;
CREATE INDEX idx_vocab_runs_member_best ON vocab_runs (member_id, correct_count DESC, max_combo DESC);
CREATE INDEX idx_vocab_runs_member_mode_best ON vocab_runs (member_id, mode, correct_count DESC, max_combo DESC);
ALTER TABLE vocab_runs DROP COLUMN language;

DELETE FROM words WHERE language <> 'en';
DROP INDEX idx_words_lang_difficulty;
CREATE INDEX idx_words_difficulty ON words (difficulty) WHERE enabled;
ALTER TABLE words DROP CONSTRAINT words_lang_word_reading_key;
ALTER TABLE words ADD CONSTRAINT words_word_key UNIQUE (word);
ALTER TABLE words DROP CONSTRAINT words_ja_reading_required;
ALTER TABLE words
    DROP COLUMN language,
    DROP COLUMN reading,
    DROP COLUMN accepted_readings;
