# Rust Axum API Server

Rust + Axum 網頁 API 伺服器，部署於 `https://kawa.homes`。

## 功能

- JWT 驗證（admin 登入 / 登出 / token refresh / Redis session）
- OAuth 登入（Google / GitHub / LINE）+ member token refresh
- WebSocket 即時推送（broadcast channel + 逐連線 sender）
- 部落格 CRUD
- HackMD 筆記同步
- Runtime 設定管理（admin 頁面熱更新，不需重啟）
- 股票資料（全市場行情、庫藏股計畫、股價變動追蹤）
- 圖片上傳 / 管理（本機儲存）
- 使用者 / 角色 / 權限管理
- 投資組合管理（member 持股 CRUD）
- 排班（roster）
- 排程 job（cron）

## API 路由

| 前綴 | 說明 |
|------|------|
| `GET /` | health check |
| `/admin/auth` | admin 登入、me、token refresh |
| `/admin/users` | 使用者管理 |
| `/admin/roles` | 角色管理 |
| `/admin/permissions` | 權限清單 |
| `/admin/settings` | Runtime 設定（讀取 / 更新） |
| `/admin/audit_logs` | 操作稽核紀錄 |
| `/oauth` | member OAuth 登入（Google / GitHub / LINE）、token refresh |
| `/members` | member 管理 |
| `/member/portfolio` | member 投資組合 CRUD、即時損益總覽、歷史價格 / 還原成本（需 Bearer token） |
| `/blogs` | 部落格 CRUD |
| `/notes` | HackMD 筆記 tags / lists |
| `/stocks` | 股票資料查詢、pending change 管理 |
| `/ws` | WebSocket 連線、線上清單、點對點訊息 |
| `/roster` | 排班 |
| `/images` | 圖片上傳 / 刪除 / 清單 |
| `/logs` | 操作紀錄 |
| `/uploads/*` | 本機靜態檔案 |
| `/tools` | 工具 |

## 排程 Job

| Job | 週期 | 說明 |
|-----|------|------|
| `ConsumePendingStockChangeJob` | 每分鐘 | 消費一筆 pending stock_change，查詢 TWSE 股價 |
| `FetchHistoricalClosingPricesJob` | 每分鐘 | 補缺起始日收盤價 |
| `CleanupUnusedImagesJob` | 每小時 | 清除 status=unused 且逾時的孤立圖片 |
| `FetchNotesJob` | 每日 UTC+8 03:00 | 同步 HackMD 筆記（需 DB 設定 `hackmd_token`） |
| `FetchStockDayAllJob` | 每日 UTC 20:00 | 抓全市場行情寫入 `stock_day_all` |
| `FetchBuybackPeriodsJob` | 每日 UTC 20:00 | 抓庫藏股計畫 HTML 寫入 `stock_buyback_periods` |
| `SyncBuybackToPendingJob` | 每日 UTC 20:10 | 將 `stock_buyback_periods` 同步為 pending stock_changes |
| `CleanupStockChangeDuplicatesJob` | 每日 UTC 21:00 | 清除 end_date 與 `stock_buyback_periods` 不符的重複 stock_changes |

## 技術棧

- `axum 0.8` — router、multipart、WebSocket
- `sqlx 0.8` — async PostgreSQL + 自動 migration
- `bb8` + `bb8-redis` — Redis 連線池
- `tokio-cron-scheduler` — cron job
- `jsonwebtoken 9` — JWT
- `reqwest 0.12` — 對外 HTTP 請求
- `tower-http` — CORS、timeout、body limit（10 MB）

## 環境變數

| 變數 | 必填 | 預設值 |
|------|------|--------|
| `DATABASE_URL` | 是 | — |
| `REDIS_HOST` | 是 | — |
| `JWT_SECRET` | 是 | — |
| `APP_HOST` | 否 | `0.0.0.0` |
| `APP_PORT` | 否 | `3000` |
| `UPLOAD_PATH` | 否 | `./uploads` |
| `GOOGLE_CLIENT_SECRET` | 否 | — |
| `GITHUB_CLIENT_SECRET` | 否 | — |
| `LINE_CLIENT_SECRET` | 否 | — |

## 常用指令

```bash
bash build.sh       # Docker build
bash up.sh          # Docker Compose 啟動
```

## 在 VPS 環境中給予指定 user 角色(super_admin)的指令
```
docker exec -it database psql -U USER -d DATABASE -c "INSERT INTO user_roles (user_id, role_id) SELECT u.id, r.id FROM users u, roles r WHERE u.email = 'kawa@gmail.com' AND r.name = 'super_admin';"
```
