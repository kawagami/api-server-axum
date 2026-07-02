# deploy — kawa.homes VPS 部署編排

原獨立 repo `docker-env` 併入 monorepo 的版本：compose 集中單檔、nginx 目錄掛載、
秘密值與持久資料移出設定樹（`/srv/kawa/`）。

## 服務

| 服務 | Image | 說明 |
|------|-------|------|
| nginx | nginx:alpine | 反向代理（`kawa.homes` → frontend、`axum.kawa.homes` → backend） |
| certbot | certbot/dns-cloudflare | Let's Encrypt 自動 renew（DNS-01） |
| database | postgres:18-alpine | 資料庫 |
| valkey | valkey/valkey:alpine | Cache（Redis 相容） |
| backend | kawagami77/api-server:latest | Rust/Axum 後端 |
| frontend | kawagami77/my-next-blog:latest | Next.js 前端 |

## 邊界原則

```
~/kawa-deploy/     ← 本目錄的 rsync 快照。CI 每次部署整個覆蓋（--delete），
                      在 VPS 上直接改 = 下次部署被無聲蓋掉。要改就改 repo。
/srv/kawa/         ← 秘密值 + 持久資料。部署永遠不碰。
├── env/           ← kawa.env（全站唯一 env 檔，backend/frontend/database 三容器共用）
│                     + cloudflare.ini（certbot 用，格式限制沒法併）。範例見 env.example/
├── uploads/       ← 後端上傳檔案
├── torrents/      ← torrent 下載檔案
└── dbdata/        ← postgres 資料（PG 18 佈局：實際 cluster 在 dbdata/18/docker/）
```

## CI 部署（日常）

- 改 `deploy/**` → `deploy.yml`：scp 到 staging → `compose config` 驗證 → rsync 覆蓋 `~/kawa-deploy` → `compose up -d` → `nginx -t` → reload。
- 改 `backend/**` / `frontend/**` → 各自 workflow build image 後 SSH：`cd ~/kawa-deploy && docker pull … && docker compose up -d`。
- 三條 deploy 共用 `concurrency: vps-deploy`，序列化不撞車。

## 全新機器 bootstrap

從零把整站架在一台新 VPS 上的流程。

### 0. 前提

- 新 VPS：建使用者 + SSH 金鑰、裝 docker（含 compose plugin）與 rsync、使用者加入 `docker` 群組
- Cloudflare DNS：`kawa.homes`、`*.kawa.homes` 指向新機 IP；SSL/TLS 模式 **Full (Strict)**
- 若新機**取代**舊機：GitHub secrets 更新 `VULTR_HOST` / `VULTR_USERNAME` / `SSH_PRIVATE_KEY`

### 1. 建持久層

```bash
docker run --rm -v /srv:/srv alpine sh -c "mkdir -p /srv/kawa/env && chown -R $(id -u):$(id -g) /srv/kawa"
```

照 `env.example/kawa.env` 建 `/srv/kawa/env/kawa.env`。**新站是產新值不是抄舊值**：
`JWT_SECRET` 產新隨機字串、`POSTGRES_PASSWORD` 自訂（空庫，首次啟動用它初始化，
`DATABASE_URL` 內密碼要同步）、OAuth secrets 沿用既有 app。
`cloudflare.ini` 放好後 `chmod 600`。

### 2. 部署設定、起服務

```bash
# 本機
rsync -av deploy/ 新VPS:~/kawa-deploy/
# VPS
cd ~/kawa-deploy && docker compose up -d
```

此時 **nginx 起不來是正常的**（憑證還不存在）。

### 3. 發憑證

```bash
bash shells/issue-cert.sh && docker compose restart nginx
```

### 4. 建第一個 admin（手動 SQL）

migration 啟動時自動跑完，`roles` / `permissions` 有 seed，但 **users 沒有**，
而 `POST /admin/users` 在認證牆後 — 第一個帳號只能手動塞：

```bash
# 產 bcrypt hash
docker run --rm python:3-alpine sh -c "pip -q install bcrypt && python -c \"import bcrypt;print(bcrypt.hashpw(b'你的密碼', bcrypt.gensalt()).decode())\""

docker exec -it database psql -U kawa -d kawa -c "
  INSERT INTO users (email, password) VALUES ('you@example.com', '<上面的hash>');
  INSERT INTO user_roles (user_id, role_id)
    SELECT u.id, r.id FROM users u, roles r
    WHERE u.email = 'you@example.com' AND r.name = 'super_admin';
"
```

### 5. 後台補 runtime 設定

登入 `/admin` → settings：OAuth client id / redirect URL、SMTP 帳密等
（存 `app_settings` 表，migration 有預設值，OAuth / SMTP 要填真值才能用）。

### 帶資料搬家的變體

不是空站而是搬家：跳過步驟 4–5，舊機停機後把整個 `/srv/kawa`
（env + uploads + torrents + dbdata）rsync 到新機同路徑；
憑證在新機重發（DNS-01 不依賴舊機）比搬 volume 簡單。

## 一次性升級 runbook：PG 17 → 18 + migration squash（2026-07-02 準備）

兩件事共用同一停機窗口：
- **PG 17 → 18**：資料目錄格式跨大版本不相容，走 dump/restore；掛載點同步改為
  PG 18 image 的新佈局（`/var/lib/postgresql`）。
- **migration squash**：`backend/migrations/` 60 個 migration 壓成單一
  `20260702000000_baseline`（schema 為兩庫 pg_dump diff 驗證過的等價版本，
  含 roles / permissions / role_permissions / app_settings 種子）。既有 DB 的
  `_sqlx_migrations` 有 60 筆舊紀錄，新後端只認 baseline 一筆，**必須手動改表**，
  否則新舊 image 都起不來。

**先不 push 這批變更**，照順序做完再 push（同舊 docker-env 切換的做法）。

### 1. 備份（站台不中斷）

```bash
# VPS 上
docker exec database pg_dump -U kawa -d kawa > ~/kawa-pg17-dump.sql
head -3 ~/kawa-pg17-dump.sql && wc -l ~/kawa-pg17-dump.sql   # 確認 dump 有內容
```

### 2. 停站、換資料目錄（停機開始）

```bash
# VPS 上
cd ~/kawa-deploy && docker compose down

# 舊 PG17 資料目錄整個改名保留（root 建的，透過容器搬）
docker run --rm -v /srv/kawa:/d alpine mv /d/dbdata /d/dbdata-pg17

# 本機把新 compose（PG18）傳上去
rsync -av deploy/ VPS:~/kawa-deploy/
```

### 3. 起 PG 18、灌回資料

```bash
# VPS 上：只起 database，等 healthy（首次啟動用 kawa.env 的 POSTGRES_PASSWORD 初始化空庫）
cd ~/kawa-deploy && docker compose up -d database
docker compose ps database    # 等 healthy

docker exec -i database psql -U kawa -d kawa -v ON_ERROR_STOP=1 < ~/kawa-pg17-dump.sql
```

### 4. 改 `_sqlx_migrations` 為 baseline 單筆

restore 回來的是 60 筆舊紀錄；換成 baseline 一筆（checksum = baseline.up.sql 的 SHA-384，
換行後校驗：`sha384sum backend/migrations/20260702000000_baseline.up.sql`）：

```bash
docker exec database psql -U kawa -d kawa -c "
  BEGIN;
  TRUNCATE _sqlx_migrations;
  INSERT INTO _sqlx_migrations (version, description, installed_on, success, checksum, execution_time)
  VALUES (20260702000000, 'baseline', now(), true,
          decode('6e6e89d969f1966ec0b7ca50347f119b23dd09bcf3df1ec38f1e6d2a16a3110e5bbaabd1d7083b7f7cf3f2ab34174ec9','hex'), 0);
  COMMIT;
"
```

### 5. 起全站、push、等 CI 換後端

```bash
# VPS 上
docker compose up -d
# 舊 backend image 會因 migration 對不上啟動失敗直接退出 —— 預期行為，
# 前台/nginx 先恢復，API 等 CI 佈上新 image
```

本機 push 這批 commit → `backend.yml` test/build/deploy 跑完（約數分鐘）後 backend 恢復。
（`deploy.yml` 也會觸發重 rsync 一次同樣的 compose，無妨。）

### 6. 驗證與善後

```bash
docker compose ps            # database healthy、backend Up
curl -sI https://axum.kawa.homes | head -1
docker exec database psql -U kawa -d kawa -tAc "SELECT version FROM _sqlx_migrations"   # 只有 20260702000000
```

- 本地開發 DB：砍掉重建（`cargo run` 會自動套 baseline），或比照步驟 4 修 `_sqlx_migrations`。
- 站台穩定幾天後刪 `/srv/kawa/dbdata-pg17` 與 `~/kawa-pg17-dump.sql`。

**回滾**（push 前）：`docker compose down` → `docker run --rm -v /srv/kawa:/d alpine sh -c "rm -rf /d/dbdata && mv /d/dbdata-pg17 /d/dbdata"` → 本機 rsync 舊版 deploy/ → `up -d`。
**回滾**（push 後）：同上，另把 backend image 釘回舊 `:<sha>` tag。

## 一次性切換 runbook（從舊 docker-env 遷移；已於 2026-07-02 完成，留檔參考）

**先不 push 這批變更**，照順序做完、站台確認活著再 push。

### 1. 前置準備（站台不中斷）

```bash
# VPS 上（kawa 無 sudo 密碼沒關係——在 docker 群組即可，用容器以 root 建目錄）
docker run --rm -v /srv:/srv alpine sh -c "mkdir -p /srv/kawa/env && chown -R $(id -u):$(id -g) /srv/kawa"
ls -ld /srv/kawa   # 應顯示擁有者為自己

# 建 /srv/kawa/env/kawa.env（照 env.example/kawa.env 的格式）：
# 把舊 docker-env/api-server-axum/.env 和 next-blog/.env.production 的內容合併貼進去，
# 兩處要改：
# - API_URL 改為 http://backend:3000（內網 hostname 跟著 compose service 名改了）
# - 補 POSTGRES_PASSWORD=kawa（先沿用舊值；既有 dbdata 不吃新密碼，
#   之後要換密碼需另外 ALTER USER 並同步 DATABASE_URL）
# JWT_SECRET 兩個舊檔都有、值相同，只留一行
cp docker-env/nginx/cloudflare.ini /srv/kawa/env/cloudflare.ini && chmod 600 /srv/kawa/env/cloudflare.ini
```

```bash
# 本機把 deploy/ 傳上去（首次手動，之後交給 CI）
rsync -av deploy/ VPS:~/kawa-deploy/
```

### 2. 切換（停機約 1–2 分鐘）

```bash
# VPS 上
cd ~/docker-env && docker-compose down

# 資料搬到 /srv/kawa（舊 dbdata 在 ~/dbdata，即 compose 裡的 ../dbdata）。
# 這三個目錄是 docker 以 root 建的，kawa 直接 mv 會 Permission denied
# （Linux 換父目錄需對目錄本身有寫入權），所以透過容器以 root 搬；
# 掛整個 / 讓 rename 在同一掛載點內瞬間完成。搬完「不要」chown 它們。
docker run --rm -v /:/host alpine sh -c "
  mv /host/home/kawa/docker-env/api-server-axum/uploads  /host/srv/kawa/uploads &&
  mv /host/home/kawa/docker-env/api-server-axum/torrents /host/srv/kawa/torrents &&
  mv /host/home/kawa/dbdata                              /host/srv/kawa/dbdata &&
  echo MOVED_OK
"

# 憑證 volume 換 project 前綴（docker-env_letsencrypt → kawa_letsencrypt）
docker volume create kawa_letsencrypt
docker run --rm -v docker-env_letsencrypt:/from -v kawa_letsencrypt:/to alpine cp -a /from/. /to/

# 起新 stack
cd ~/kawa-deploy
docker compose config --quiet && docker compose up -d
```

### 3. 驗證

```bash
docker compose ps                     # 六個服務都 Up、database healthy
curl -sI https://kawa.homes | head -1
curl -sI https://axum.kawa.homes | head -1
# 瀏覽器：前台登入（驗 JWT_SECRET 共用正確）、後台登入、WS 頁面
```

### 4. 善後

- 站台穩定後：本機 push 這批 commit（CI 三條 workflow 從此接管）。
- 舊 `~/docker-env` 目錄與 `docker-env_letsencrypt` volume 保留幾天當退路，確認無事再刪；
  GitHub 的 docker-env repo 封存（archive）。

**回滾**（新 stack 起不來時）：`cd ~/kawa-deploy && docker compose down`，資料 `mv` 回原位，
`cd ~/docker-env && docker-compose up -d`。

## 首次憑證（新機才需要）

`bash shells/issue-cert.sh`（憑證進 `kawa_letsencrypt` volume，之後 certbot 容器每 12h 自動 renew、nginx 每 6h reload）。
