# kawa.homes · 個人網站 Monorepo

個人網站的全端 monorepo:**Rust / Axum 後端** + **Next.js 前端** + **部署編排**(docker-compose / nginx / certbot),合併於單一 repo。

線上站台:[kawa.homes](https://kawa.homes)(API:`axum.kawa.homes`)

---

## 專案結構

```
.
├── backend/            # Rust / Axum API 伺服器
├── frontend/           # Next.js 前台 + 後台
├── deploy/             # VPS 部署編排(compose / nginx / certbot),詳見 deploy/README.md
└── .github/workflows/
    ├── backend.yml     # 後端 CI:paths 過濾 backend/**、context ./backend
    ├── frontend.yml    # 前端 CI:paths 過濾 frontend/**、context ./frontend
    └── deploy.yml      # 編排 CI:paths 過濾 deploy/**,rsync 設定到 VPS 並套用
```

各子專案的細節見各自目錄下的說明;本 README 只講整體與整合。

## 技術棧

| | 後端 `backend/` | 前端 `frontend/` |
|---|---|---|
| 語言 / 框架 | Rust 2021 · Axum 0.8 · Tokio | Next.js 16 · React 19 (App Router) |
| 資料 | PostgreSQL(sqlx)· Redis/Valkey(bb8) | — |
| 其他 | JWT 認證 · tokio-cron 排程 · WebSocket · librqbit | Tailwind CSS · next-intl(zh-TW/zh-CN/en)· JWT(jose) |
| 套件管理 | cargo | pnpm |

## 主要功能

- **部落格**:文章 CRUD、標籤、Markdown 閱讀頁 + TOC
- **後台管理**:RBAC 權限(user/role/permission)、稽核紀錄、站台設定熱更新、主題切換
- **會員系統**:OAuth 登入(Google / GitHub / LINE)、投資組合、記帳、統一發票登錄對獎、大樂透/威力彩選號對獎
- **對戰遊戲平台**(WebSocket):象棋、五子棋、暗棋、西洋棋、圍棋、阿瓦隆、農場經營
- **股票**:庫藏股追蹤、收盤價統計、每日行情
- **工具**:Torrent 下載、HackMD 筆記同步、不重複到訪統計

## 三個部分如何串接

整合邊界是 **Docker Hub image + HTTP/WS API**,不是直接 import 原始碼:

```
backend/  ──CI build──▶ kawagami77/api-server:latest ───┐
frontend/ ──CI build──▶ kawagami77/my-next-blog:latest ─┤
                                                         ▼
deploy/ ──CI rsync──▶ VPS ~/kawa-deploy(compose + nginx + certbot)
                      VPS /srv/kawa(秘密值 env + uploads/torrents/dbdata,不進 git)
```

1. **Docker 映像**:前後端各自 build image 推到 Docker Hub;image 名稱是契約,改名要同步 `deploy/docker-compose.yml`。
2. **HTTP / WebSocket**:前端用 `API_URL` / `WS_URL` 環境變數連後端,URL 不寫死。
3. **編排**:`deploy/` 的 compose + nginx vhost(`kawa.homes` → frontend、`axum.kawa.homes` → backend)。秘密值只存在 VPS `/srv/kawa/env/kawa.env`(全站唯一 env 檔,三容器共用,`JWT_SECRET` 只出現一次)。

## 本地開發

前後端可獨立啟動;前端靠 `API_URL` / `WS_URL` 指向本地或遠端後端。

```bash
# 後端(讀 backend/.env)
cd backend && cargo run

# 前端(讀 frontend/.env.local)
cd frontend && pnpm install && pnpm dev
```

各自需要的環境變數見 `backend/.env.example` 與 `frontend/.env.example`。

## CI / 部署

- **Path-based CI**:改 `backend/**` 只觸發 `backend.yml`、改 `frontend/**` 只觸發 `frontend.yml`、改 `deploy/**` 只觸發 `deploy.yml`(同步編排設定,不重 build image),互不重複執行。
- 前後端 workflow 拆成 `test`(後端 clippy + cargo test、前端 tsc --noEmit)、`build`(build+push image)與 `deploy`(SSH VPS)三段;test 不過不會 build 與部署。三條的 `deploy` 共用 `concurrency: vps-deploy`,**build 並行、部署序列化**,避免同時動 VPS 撞車。
- image 同時推 `:latest`(部署契約)與 `:<commit sha>`(回滾用)。
- **push `master` = 直接上 production**(test → build image → SSH VPS → pull + 重啟)。
- 環境變數一律 **runtime 注入**,image 內不烤設定值;秘密值只存在 VPS `/srv/kawa/env/`(範例見 `deploy/env.example/`)。
