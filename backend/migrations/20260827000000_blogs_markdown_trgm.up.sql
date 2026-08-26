-- 讓公開/後台列表的關鍵字搜尋（markdown ILIKE '%q%'）走索引，取代對 blogs 的循序掃描。
--
-- 2026-08-27 實測（2000 篇、markdown 各約 1.5KB、表 3MB）：
--   選擇性高的關鍵字   Seq Scan 2.3ms → Bitmap Index Scan 0.13ms（約 17×）
--   命中全部文章的詞   ~28ms，**索引無效**（row 真的全中，索引只是換條路徑取同樣的資料）
--
-- ⚠ 所以這個索引**不是** DoS 防線。列表端點的 count 那一半無法靠 LIMIT 提早結束，
--   一個 `?q=常見詞` 的請求就是 ~60ms 的 PG CPU。擋那件事的是 nginx 的 `search`
--   limit_req zone（帶 ?q= 才計數，見 deploy/nginx/nginx.conf）。這個索引管的是
--   「正常搜尋要多快」。
--
-- ⚠ 只有**三字以上**的關鍵字吃得到：pg_trgm 對 LIKE/ILIKE 樣式取的是樣式內部完整的
--   三連字，`%ab%` 抽不出任何一個，planner 會退回 Seq Scan。中文兩字詞（很常見）就落在
--   這個洞裡 —— 實測 `%後端%` 確實是 Seq Scan、`%罕見詞%` 才走索引。
--
-- ⚠ 不用 CONCURRENTLY：sqlx 的 migration 跑在交易內，CONCURRENTLY 在交易內直接報錯。
--   blogs 是小表（個人站規模），一般建索引的鎖時間可忽略。
--
-- ⚠ GIN 有 pending list：大量寫入後第一次查詢可能仍走 Seq Scan，要等一次索引掃描或
--   autovacuum 把 pending list 併進去才會穩定用上索引（測試時踩到過，不是設定錯）。
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS idx_blogs_markdown_trgm ON blogs USING GIN (markdown gin_trgm_ops);
