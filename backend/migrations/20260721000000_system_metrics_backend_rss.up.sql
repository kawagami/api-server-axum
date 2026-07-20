-- backend 行程自身 RSS(MB),與整機 mem_used_mb 分開追蹤,用以辨別記憶體爬升是後端還是 PG/Redis/前端。
-- 既有 row 無此量測,補 0(前端視為缺值)。
ALTER TABLE public.system_metrics
    ADD COLUMN backend_rss_mb integer NOT NULL DEFAULT 0;
