-- 下載開始時間（拿到 metadata、進入 downloading 的那一刻），供 /admin/torrents 算實際下載耗時
-- （completed_at - started_at，不含排隊與 metadata 解析）。
-- 重啟續傳會再走一次 set_downloading_metadata，所以只在 NULL 時寫入；手動重跑（reset_pending）清回 NULL。
-- 既有列維持 NULL（前端退回以 created_at 計）。
ALTER TABLE torrents ADD COLUMN started_at timestamp with time zone;
