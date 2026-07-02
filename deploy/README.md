# deploy — kawa.homes VPS 部署編排

原獨立 repo `docker-env` 併入 monorepo 的版本：compose 集中單檔、nginx 目錄掛載、
秘密值與持久資料移出設定樹（`/srv/kawa/`）。

## 服務

| 服務 | Image | 說明 |
|------|-------|------|
| nginx | nginx:alpine | 反向代理（`kawa.homes` → frontend、`axum.kawa.homes` → backend） |
| certbot | certbot/dns-cloudflare | Let's Encrypt 自動 renew（DNS-01） |
| database | postgres:17-alpine | 資料庫 |
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
└── dbdata/        ← postgres 資料
```

## CI 部署（日常）

- 改 `deploy/**` → `deploy.yml`：scp 到 staging → `compose config` 驗證 → rsync 覆蓋 `~/kawa-deploy` → `compose up -d` → `nginx -t` → reload。
- 改 `backend/**` / `frontend/**` → 各自 workflow build image 後 SSH：`cd ~/kawa-deploy && docker pull … && docker compose up -d`。
- 三條 deploy 共用 `concurrency: vps-deploy`，序列化不撞車。

## 一次性切換 runbook（從舊 docker-env 遷移）

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
