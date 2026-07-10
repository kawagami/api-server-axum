-- 單字闖關週期排行榜:週/月榜掃 language + ended_at 區間聚合用
CREATE INDEX idx_vocab_runs_lang_ended ON vocab_runs (language, ended_at);
