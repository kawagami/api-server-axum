-- logs 表補請求上下文。
-- 原本 `tracing::error!(?self, "System error occurred")` 進 DB 只留 message 那一行,
-- `?self`(真正的錯誤細節)與 span 上的 request_id / method / path 全被 DbLogLayer 丟掉,
-- 只剩 stdout(docker logs)看得到 —— 等於「回傳了 request_id 卻查不到它」。
ALTER TABLE public.logs ADD COLUMN IF NOT EXISTS request_id text;
ALTER TABLE public.logs ADD COLUMN IF NOT EXISTS fields jsonb;

-- 拿 request_id 撈整個請求的所有 log 是查線上問題的主要動作。
-- 非請求路徑的 log(job / 啟動期)這欄為 NULL,故用 partial index 讓索引只長在有值的列上。
CREATE INDEX IF NOT EXISTS idx_logs_request_id
    ON public.logs (request_id) WHERE request_id IS NOT NULL;
