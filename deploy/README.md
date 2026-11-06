# deploy — kawa.homes VPS 部署編排

原獨立 repo `docker-env` 併入 monorepo 的版本：compose 集中單檔、nginx 目錄掛載、
秘密值與持久資料移出設定樹（`/srv/kawa/`）。

## 服務

| 服務 | Image | 說明 |
|------|-------|------|
| nginx | nginx:alpine | 反向代理（`kawa.homes` → frontend、`api.kawa.homes` → backend，舊名 `axum.kawa.homes` 同 vhost 保留為 alias，其 `/uploads/` 301 到 media）＋ `media.kawa.homes` 直出上傳圖片（`root /srv/kawa/uploads`，掛唯讀 volume） |
| certbot | certbot/dns-cloudflare | Let's Encrypt 自動 renew（DNS-01） |
| database | postgres:18-alpine | 資料庫 |
| valkey | valkey/valkey:alpine | Cache（Redis 相容）。**無持久化，重啟即清空** —— 見下方說明 |
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

**valkey 不在這個清單裡 —— 它刻意沒有持久化**，資料全在記憶體，容器重啟就清空。
這是有意識的取捨，代價要知道：

- **member refresh token（TTL 30 天）也會沒了 → valkey 重啟等於把所有會員登出**，下次操作需重新登入。
- **後台也一樣，而且是立刻**：admin 的登入 session（`user:login:{id}`，TTL 1h）是 `middleware::auth`
  每個請求都查的唯一判斷，`/admin/auth/refresh` 本身也在認證牆後 —— key 沒了下一個請求就 401。
- 會中斷但不登出：進行中的單字闖關（run 狀態，30m）、進行到一半的 passkey 登入 / 註冊（challenge，5m）、
  oauth state（5m）、ws ticket（30s），重來一次即可。
- 掉了無感：身分 / 權限快取（1h）、持股的月收盤價與除權息快取，會自動回頭查 DB（或重抓 TWSE）重建。

所以 `docker compose restart valkey`、整機重開、image 更新都會踢人。哪天覺得代價太大，
就掛 `/srv/kawa/valkey:/data` + `--appendonly yes`。

> `--maxmemory 128mb --maxmemory-policy volatile-lru` 與 `mem_limit: 160m` 已經在
> `docker-compose.yml` 上了（本節一度寫「目前 valkey 沒有記憶體上限」，已過期）。
> 2026-08-09 實測 RSS 只有 4.9M —— 上限離用量很遠，不是限制因素。

## 日誌 rotation

Docker 預設的 json-file **沒有上限**，只會單向長大到把磁碟塞滿（nginx 的 access_log 走
`/dev/stdout`，每個請求一行，是最大的產出者）。compose 用三個 YAML anchor 依產量分級：

| 額度 | 服務 | 上限 |
|---|---|---|
| `x-logging-heavy` | nginx | 20m × 5 |
| `x-logging-normal` | backend / frontend | 10m × 3 |
| `x-logging-quiet` | database / valkey / certbot | 10m × 2 |

**與 `/admin` 觀測頁無關。** 觀測頁的日誌是後端 tracing 的 `DbLogLayer` 直接寫 postgres
`logs` 表（只收 WARN/ERROR，保留 14 天由 `cleanup_observability` job 管），從頭到尾沒碰 Docker。
這裡限制的是 `docker logs` 能往回看多久 —— backend 是 distroless、`exec` 進不去，
**INFO 級訊息只存在 `docker logs`**，所以額度刻意不開太小。rotation 後 `docker logs`
行為不變（會跨輪替檔讀，`--since` / `-f` 照常），只是歷史被 `max-size × max-file` 封頂。

⚠ `logging:` 只在**容器建立時**生效：改額度後下次 `docker compose up -d` 會因 config hash
變動而 recreate 那些 service（等於全站滾動重啟一次，**valkey 重啟＝會員被登出**，見上一節）。
既有的 json log 檔不會被回溯截斷，舊檔隨舊容器一起消失。查目前佔用：

```bash
sudo du -sh /var/lib/docker/containers/*/*-json.log | sort -h | tail
```

## CI 部署（日常）

- 改 `deploy/**` → `deploy.yml`：scp 到 staging → `compose config` 驗證 → rsync 覆蓋 `~/kawa-deploy` → `compose up -d` → `docker pull nginx:alpine`（浮動 tag 不 pull 就永遠停在舊版；失敗只警告不擋部署）→ 一次性容器 `nginx -t` → `--force-recreate` nginx → 冒煙檢查（`curl https://api.kawa.homes/health`，最多重試 10 次）→ 通過才 `docker image prune --force`。
- 改 `backend/**` / `frontend/**` → 各自 workflow build image 後 SSH：`cd ~/kawa-deploy && docker pull … && docker compose up -d` → **`nginx -s reload`**（原因見下）。
- 三條 deploy 共用 `concurrency: vps-deploy`，序列化不撞車。

### `docker ps` 的 IMAGE 欄顯示一串 ID，而不是 image 名稱 —— 通常不是部署失敗

新機（kawa-deb）的 Docker 用 containerd image store（`docker info --format '{{.Driver}}'` 是 `overlayfs`），
image ID 是用 manifest 算的。只改文件的 push（例如 frontend 的 README）會讓 build 完全命中快取，產出**內容相同**的 image，
但推到 Docker Hub 後 manifest 外層的中繼資料不同，pull 下來就是**新 ID**。compose 判斷內容沒變，不會重建容器；
`:latest` 標籤已經指到新 ID，所以 `docker ps` 只能用 ID 顯示正在跑的舊 image。

2026-11-06 的 frontend 就是這樣：新舊 image 的 `Created` 一樣到奈秒，`RootFS.Layers` 也完全相同。確認方式：

```bash
docker image inspect <ps 上的 ID> --format '{{json .RootFS.Layers}}' | sha256sum
docker image inspect kawagami77/my-next-blog:latest --format '{{json .RootFS.Layers}}' | sha256sum
```

兩個一樣就什麼都不用做；不一樣才是真的沒更新到，要 `docker compose up -d --force-recreate <服務>` 再 reload nginx。

### ⚠ 換 image 後必須 reload nginx —— upstream IP 會過期

`upstream backend_app` / `frontend_app`（`conf.d/02-proxy.conf`）的 `server backend:3000` 是**靜態 hostname**，
nginx 只在載入設定時解析一次、之後永久快取那個 IP。`docker compose up -d` recreate 掉 backend / frontend 容器後 Docker 可能配到新 IP，
nginx 卻還在打舊 IP → **502**，而且只會等到 compose 裡那個 6h reload 循環才自癒（最久 6 小時）。
Docker 常把同一個 IP 配回來，所以它是間歇性的，不會每次部署都炸。

`backend.yml` / `frontend.yml` 的 deploy 因此在 `up -d` 後多跑一行
`docker compose exec -T nginx nginx -s reload` —— reload 會重新解析所有 upstream 名稱。
（`deploy.yml` 不需要，它本來就會 `--force-recreate` nginx。）

手動在 VPS 上換 image 時同理：`up -d` 完記得 reload。

## 效能量測

`deploy/scripts/perf-check.sh` —— 換 CF 方案 / 搬機房 / 改快取策略時的前後對照基準。會隨部署 rsync 到 VPS，所以兩種模式都能就地跑。

```bash
./perf-check.sh           # 使用者視角（CF 節點、TCP/TLS/TTFB、快取狀態、圖片最佳化健檢）
./perf-check.sh origin    # 在 VPS 上跑，量 origin 本身（繞過 CF）
```

2026-07-25 的基準（HiNet 固網）：

| 量測點 | 數值 |
|---|---|
| origin 處理（`/zh-TW` SSR，直打容器） | ~10ms |
| origin 經 nginx+TLS | ~30ms |
| 到 CF 邊緣（`cloudflare.com`，KHH 高雄） | **11ms** |
| 到 CF 邊緣（本站，**SIN 新加坡**） | **193ms** |

**結論：瓶頸不在 origin（只佔 3%），而是 CF 免費方案把本站的網域丟到遠端節點而非台灣（當時是新加坡；節點會漂移，見下方 10-22 複測）。** 腳本會自動比對本站與 `cloudflare.com` 的 `cf-ray` 節點代碼並標出差異 —— 那個 baseline 代表「這條線路本來碰得到的最近節點」。

2026-10-22 複測（HiNet 固網，起因是「瀏覽起來變慢」）—— **節點會漂移**，這次三個網域全落 **SJC 聖荷西**：

| 量測點 | `/health` | `/zh-TW` |
|---|---|---|
| origin 直打容器 | ~0ms | 20ms |
| origin 經 nginx+TLS（`perf-check.sh origin`） | 14.5ms | 37ms |
| client 經 CF（SJC），TTFB 中位 | **585ms** | **612ms** |

- 到邊緣 TCP 握手：本站 SJC 141ms、`cloudflare.com`（HKG）中位 72ms。SJC 握手比 7/25 的 SIN 還快，**但整體更慢**：`/health` 的 585ms ≈ TLS 完成 ~295ms + 你↔SJC 往返 141ms + SJC↔VPS 往返 ~135ms + origin 14ms —— 請求先飛美西再飛回 VPS。
- origin 佔 2.5–6%，容器 CPU ≈0、記憶體遠低於上限；同時段 `system_metrics` CPU ~2.7%、steal ≈0。**感覺變慢時先跑這支看 colo，不要先查程式。**
- 能改善的只有路由本身（Argo Smart Routing / 升級方案）；改 DNS-only 繞過 CF 會曝露 origin IP 並失去 DDoS / WAF / 快取，不採用。

以上兩次都是**舊機 first-laravel**。2026-11-06 **新機 kawa-deb（Tokyo）複測**（HiNet 固網）—— 換機**沒有改善**，
三個網域仍落 **SJC**：

| 量測點 | `/health` | `/zh-TW` |
|---|---|---|
| origin 直打容器 | ~0ms | 50ms |
| origin 經 nginx+TLS（`perf-check.sh origin`） | 25ms | 69ms |
| client 經 CF（SJC），TTFB 中位 | **584ms** | **631ms** |

- 到邊緣 TCP 握手：本站 SJC 141ms、`cloudflare.com`（KHH）中位 18ms。推算 SJC↔VPS 往返約 115–140ms
  （差在 CF 回源有沒有重用連線；舊機約 135ms）—— 新機到 SJC 的距離跟舊機差不多，**瓶頸仍是 CF 把台灣使用者路由到 SJC**，
  不是主機。換機房解決不了這件事。
- origin 比舊機慢：本機 TLS 握手 ~23ms、`/zh-TW` SSR 直打容器 50ms（舊機 20ms）。都是單次取樣，
  佔整體也只有 4–11%，不影響結論；要追的話多跑幾次，並看 `system_metrics` 的 steal。
- 容器記憶體（穩態，`docker stats`）：frontend 112M、database 98M、valkey 8M、nginx 7M、backend 7M、certbot 5M，
  合計約 238M（舊機 08-09 約 337M）。
- 圖片最佳化正常（w=64 / 256 / 1200 → 782 / 5822 / 45652 bytes，size 隨寬度變化）。

## blog 搜尋限流（`search` zone，2026-08-27）

`?q=` 的搜尋是全站最貴的公開查詢：列表端點的 `count` 那一半無法靠 LIMIT 提早結束，一個命中大量文章的常見詞 ≈ **60ms 的 PG CPU**（2026-08-27 實測，2000 篇 / 3MB 表）。general 的 30r/s 放進來就是單核吃滿。

`nginx.conf` 的 `search` zone（20r/m，burst 5）用一個 map 當 key：**沒有 `?q=` 時 key 是空字串，limit_req 對空 key 不計數**，所以同一條指令只對「帶 `q=` 的請求」生效，一般瀏覽不計數。

### ⚠ map 只看 `q=`、不看路徑 —— 會誤中的路徑要開獨立 location（2026-08-30）

`$arg_q` 是**任何**帶 `q=` 的網址，與「是不是搜尋」無關。Next.js 的圖片最佳化網址
`/_next/image?url=…&w=1920&q=75` 那個 `q=` 是**畫質**，照樣被算進 search zone —— 圖多的 blog
一次併發十幾張，第 7 張起（burst 5 + 1 個 rate token）一律 429，在瀏覽器上就是
**「原圖網址打得開、頁面上卻破圖」**（2026-08-30 實測確認，commit 879a22d 修）。

修法是把會誤中的路徑拉出獨立 location、只掛 `general`。目前已這樣處理的（都在 `conf.d/kawa.homes.conf`）：

- `location /_next/`：image 最佳化 + static 資產全段
- `location /admin`（2026-10-12；**結尾不可加斜線**，`/admin/` 會與 Next 的去尾斜線互轉成無限迴圈，10-13 踩過）：後台篩選列把 `q` 寫進網址（`hooks/useFilterUrl.ts`），而 Next 的
  server action 是 POST 到**目前頁面網址**，於是帶著關鍵字後的翻頁、開 trace、輪詢全被當搜尋計數，
  連點十次上下就開始 200 / 429 交錯（burst 5 用完後每 3 秒才補 1 次）。DevTools 實測確認。未登入的請求在 `proxy.ts` 就被擋，拉出來不會變成繞過搜尋限流的門。
**日後前端新增任何帶 `q=` 卻不是搜尋的路徑，要照這個模式再開一個 location**，
否則症狀是靜默的 429 破圖，不會有錯誤訊息。拉出來的 location 別自帶 `add_header` /
`proxy_set_header`（會整組覆蓋掉 server 層那些）。

兩個 vhost 都要掛,少一個等於沒掛:

- `api.kawa.homes` 的 `location /blogs` —— 直接打 API 的
- `kawa.homes` 的 `location /` —— **blog 搜尋頁是 SSR**,`kawa.homes/zh-TW/blogs?q=` 由 Next 在伺服器端打 `http://backend:3000`,那條走內網、不經 api vhost

同一個 IP 共用一份額度(不論從哪個入口進來),這是刻意的 —— 貴的是後面那個查詢,不是入口。

驗證(2026-08-27 實測結果):

```
一般列表 x15   → 全 200        （不受 search zone 影響）
搜尋     x12   → 6 個 200 後轉 429（burst 5 + 1 個 rate token）
```

改動 `/_next/` 這類 location 後也要順手驗一次圖：連開一頁圖多的 blog，
`curl -sI 'https://kawa.homes/_next/image?url=…&w=1920&q=75'` 連打 10 次應全 200。

## 公開 API 快取（`api_cache`，2026-08-27）

nginx 對 `api.kawa.homes/blogs` 開了 proxy_cache（zone 與 map 定義在 `nginx/conf.d/02-proxy.conf`，套用在 `nginx/conf.d/api.kawa.homes.conf` 的 `location /blogs`）。

**為什麼只有這一條路徑**:前端 SSR 讀 blog 已經有 Next Data Cache（`frontend/api/blogs.ts` 的 `revalidate`），但那條打的是內網 `http://backend:3000`、**不經 nginx**。這層快取要擋的是另一個情境:有人拿 origin IP 直接洪水打公開 API —— 那時 PG（384m、1 核共用、pool 20 條）沒有任何緩衝。

規則(全部在設定檔裡有註解):

- **能不能快取由 backend 決定**:只有帶 `Cache-Control`（`backend/src/routes.rs` 的 `public_cache()`,`s-maxage=60`）的回應會被存。所以 `GET /blogs/{id}/comments` 自動不會被快取,不必在 nginx 維護排除清單。
- 帶 `Authorization` 或任何 `Cookie` → 完全繞過(不讀也不寫)。
- 帶 `?q=` → 繞過,避免任意關鍵字把 LRU 沖掉(搜尋改由上面的 `search` limit_req zone 擋量)。
- **絕不在 server 層開**:這個 vhost 絕大多數是動態端點,開在 server 層等於「預設全部可快取」,日後新端點忘了帶 `Cache-Control` 就會被靜默共用出去。最極端的案例是已移除的 `GET /tools/new_password`(全域快取＝所有人拿到同一組密碼;該功能已改由瀏覽器端產生)——端點會來會去,規則不隨它消失。

⚠️ 這一層**不會被 `updateTag('blogs')` 失效**。後台存檔後 Next 那層立刻更新,nginx / 瀏覽器最久等 60 秒。

⚠️ 快取只吃得到「重複的匿名 GET」。要製造 miss 很容易(換 `?q=`、換 `Origin` —— 後端 CORS 送 `Vary: origin`,nginx 照 Vary 分變體)。**這層擋的是流量放大與正常尖峰,不是有針對性的攻擊**;後者的正解是防火牆只放行 CF 網段(尚未做)。

### 驗證(這類設定失效時是靜默的 —— 只會一直 MISS,沒有錯誤)

`location /blogs` 會回 `X-Cache-Status`,直接看它就好:

```bash
curl -sI https://api.kawa.homes/blogs | grep -i x-cache-status   # 第二次應為 HIT
curl -sI 'https://api.kawa.homes/blogs?q=x' | grep -i x-cache-status        # 應為 BYPASS
curl -sI https://api.kawa.homes/blogs -H 'Cookie: a=b' | grep -i x-cache-status  # 應為 BYPASS
```

沒有 `X-Cache-Status` 這個 header ＝ 請求根本沒進到那個 location。一直 MISS ＝ backend 沒送 `Cache-Control`,或快取目錄不可寫（`docker exec nginx ls -ld /var/cache/nginx/api`,並看 `docker logs nginx` 有沒有 `Permission denied`）。

> ⚠️ 這兩節（`search` zone 與 `api_cache`）**同時動了 `nginx.conf` 與 `conf.d/`**，所以
> **必須 recreate nginx，不能只 reload** —— `limit_req_zone search` 在 `nginx.conf` 裡
> （`proxy_cache_path` 則在 `conf.d/02-proxy.conf`，那是目錄掛載、reload 吃得到），
> 只 reload 的話容器吃到的是舊的 `nginx.conf`，會報 `zero size shared memory
> zone "search"`（看起來像語法錯，其實是設定檔沒更新）。理由見下一節。
> CI 的 `deploy.yml` 本來就會 `--force-recreate` nginx，走 CI 不必額外處理。

### ⚠ nginx.conf 是單檔 bind mount —— 改它必須 recreate 容器

`docker-compose.yml` 把 `./nginx/nginx.conf` 以**單一檔案**掛進容器，而單檔 bind mount 綁的是 **inode**。rsync 的預設行為是「寫暫存檔再 rename」，會換掉 inode，**執行中的容器因此看不到新的 `nginx.conf`**。相對地 `./nginx/conf.d` 是**目錄**掛載，目錄內的檔案變動會正常反映。

後果是新舊混用，而且錯誤訊息會指向錯的地方 —— 例如在 `conf.d` 新增 `limit_req zone=auth`、同時在 `nginx.conf` 新增對應的 `limit_req_zone`，容器只吃到前者，報的卻是 `zero size shared memory zone "auth"`，看起來像語法錯，實際是設定檔沒更新（2026-07-25 實際踩過）。

因此 `deploy.yml` 的做法是：

1. **用一次性容器驗證**，掛載當下磁碟上的檔案（不是執行中容器裡那份舊的）。借 `--network container:nginx` 取得 compose 網路，否則 upstream 的 `server backend:3000` 在載入階段就 host not found。
2. 通過後 **`docker compose up -d --force-recreate --no-deps nginx`** —— 只有 recreate 會重新解析 bind mount，`restart` 與 `nginx -s reload` 都不會。

手動改 VPS 上的 nginx 設定時同理：動到 `nginx.conf` 就得 recreate，只動 `conf.d` 才能 reload 了事。

## 主機 IPv6：必須整台關掉（2026-08-09）

Vultr 的機器（舊機 first-laravel、新機 kawa-deb 都是）建機時沒勾 Public IPv6，就**沒有可用的 IPv6，但 IPv6 stack 開著**
（`ip -6 addr` 只有 `fe80::`、`ip -6 route` 沒有 `::/0`）。glibc / Go 因此以為有 v6，對外連線先試 AAAA 再撞牆 —— SMTP 寄信中過。
（當時一併歸因的 OAuth 與 `docker pull` 事後查明關 IPv6 修不到：前者見下節「DNS 抖動」、後者見下方
「關了也還在的東西」。）舊機於 2026-08-09 08:46 (UTC) 關閉；新機 kawa-deb 在 2026-11-04 建機時、裝 Docker 之前就關了。

### 修法與**不可顛倒的順序**

1. **先**拆掉 `nginx/conf.d/*.conf` 的 `listen [::]` 並部署（已完成）。少了這步，第 2 步之後 nginx
   `socket() [::]:80 failed (97: Address family not supported by protocol)` 起不來，**全站掛掉**。
2. **再**在宿主機關閉 IPv6：

```bash
sudo tee /etc/sysctl.d/99-disable-ipv6.conf <<'EOT'
net.ipv6.conf.all.disable_ipv6 = 1
net.ipv6.conf.default.disable_ipv6 = 1
EOT
sudo sysctl --system
sudo systemctl restart docker   # daemon 的 IPv6 能力是行程啟動時探測一次，不重啟不會生效
```

> `systemctl restart docker` 會重啟所有容器，比照一次完整部署安排時間。

### 關了也還在的東西

- **dockerd 仍會撥 AAAA**：它是 Go，resolver 不套 glibc 的 `AI_ADDRCONFIG`，關 IPv6 只把 errno 從
  `EADDRNOTAVAIL` 換成 `ENETUNREACH`。沒有乾淨的根治手段 ⇒ `backend.yml` / `frontend.yml` 的
  `docker pull` **重試 3 次**（退避 5 / 10 秒）。
- compose 裡 backend 的 `sysctls: net.ipv6.conf.*.disable_ipv6=1` **不是解法**（實測沒修好任何東西），
  只當衛生與「宿主機設定被還原」時的備援。

## 對外連線 DNS 抖動（現況與規則，2026-08-12 定案）

**症狀**：backend 對外請求偶發在連線前失敗，單發、重新 resolve 就通。三種 errno 是同一件事：

| errno | 意義 |
|---|---|
| `os error 99`（`EADDRNOTAVAIL`） | 解析只回 AAAA、沒有 A → 沒有 v4 可退 |
| `Name or service not known`（`EAI_NONAME`） | A / AAAA 都沒拿到 |
| `No address associated with hostname`（`EAI_NODATA`） | 回了但沒有可用 family |

**研判根因**：Docker 內嵌 DNS（127.0.0.11）把一次解析拆成 A 與 AAAA 兩個上游查詢，掉一個就只回另一個。
最容易掉的時機是容器剛重啟（冷快取）與 **cron tick 的 `:00.x`**（多支 job 同秒並行解析）。
「內嵌 DNS 為什麼會掉」本身仍未解釋。

**已上線的防護**（改任何一項前先讀 `INCIDENTS.md` 對應段落）：

- `docker-compose.yml` backend 的 `dns_opt: [single-request-reopen, timeout:2, attempts:3]`（只對 glibc 有效，distroless/cc-debian12 成立）
- `utils/reqwest.rs::send_retrying`：**4 次 / 指數 250-500-1000ms**，重試會重新 resolve —— 對症的解
- `services/email.rs`：**3 次 / 線性 300-600ms**
- 兩者都只重試「請求還沒送達對方」的失敗；已回狀態碼、或信已進 SMTP 對話的一律不重試（免得通知寄兩次）
- 部署腳本 `docker pull` 重試 3 次（見上節）

**監控**：`scripts/kawa-logs logs -q "對外請求暫時性失敗"`。要盯的字串是 **「第 3/4 次」**
（舊預算的「第 2/3 次」不會再出現，拿它當條件永遠是零）。出現了代表重試吃不下，才往宿主機
`/etc/resolv.conf` 與內嵌 DNS 上游查。

### 不要做（都試過或推翻過，理由在 `INCIDENTS.md`）

- **不要加 `dns: [8.8.8.8, …]`**：它要修的「死的 v6 上游」是 append 在清單尾端的 fallback，正常路徑碰不到；
  加了還會讓人以為那條線索處理過了。
- **不要用 reqwest `local_address(0.0.0.0)` 只走 IPv4**：AAAA-only 的答案濾完是空清單，照樣失敗。
- **不要隨手放寬 `send_retrying` 預算**：`services/oauth.rs` 的 member 登入路徑也走它，
  抖動時 GitHub 登入最壞已多等 5.25 秒。
- **不要用 alpine 測 DNS 行為**：musl 會自己濾 AAAA，與 backend 的 glibc 不同。要測用
  `debian:12-slim` 並 `--network container:backend`（壓測腳本見 `INCIDENTS.md`）。

**下一步（未做）**：reqwest 的 `hickory-dns` feature（純 Rust resolver，自帶快取）。換之前要確認
`database` / `valkey` 仍由 127.0.0.11 解析。

## 新機 OS 準備（Debian 13，2026-11-01 定案）

bootstrap 與搬家 runbook 共用的主機層步驟。主機只負責「開機、跑 docker、收安全更新」，其餘都在容器裡。

### 為什麼換機、為什麼選 Debian

- 舊機（`first-laravel`，2026-11-04 已被 kawa-deb 取代）是 **Ubuntu 20.04 + kernel `5.4.0-170`**。20.04 的標準支援 2025-05 已結束，
  而且**沒接 Ubuntu Pro**（2026-11-01 `pro status` 確認）⇒ 核心與大部分套件從那之後**沒有任何安全更新**。
  教訓不在發行版本身，而是**沒開自動更新、沒人盯 EOL**。
- 選 **Debian 13（trixie）**：1G 記憶體下沒有 snapd、預設服務少；核心安全修補快（2026-10 的
  CVE-2026-98432 是 Debian 先出修補，Ubuntu 落後）；apt / systemd 與舊機相同，runbook 不必改寫。
  代價：沒有免費 livepatch（核心更新靠下方的自動重開機）、支援期比 Ubuntu LTS + ESM 短。
- **EOL：一般安全支援約到 2028-08，LTS 約到 2030-06。** 到期前一年就開始規劃換機。

### 1. 使用者與 SSH

```bash
# root 身分（新機若預設只有 root）
apt update && apt install -y sudo
adduser --uid 1001 kawa && usermod -aG sudo kawa   # uid 對齊舊機，見下方
```

- 本機公鑰放進 kawa 的 `~/.ssh/authorized_keys`（`ssh-copy-id kawa@新機IP`）；**CI 用的 `SSH_PRIVATE_KEY`
  對應的公鑰也要放**，否則換完 secrets 後部署會連不上。
- **kawa 的 uid 要和舊機一樣**。舊機 first-laravel 的 kawa 是 **1001**，不是 1000（2026-11-04 確認），
  所以要用 `adduser --uid 1001 kawa` 建立，gid 也要對上（`id kawa`）。搬家用 `--numeric-ids` 原樣保留擁有者，
  uid 對不上的話，`/srv/kawa` 裡 kawa 擁有的檔案到新機會變成別人的。

改主機名稱時，Vultr 的 cloud-init 每次開機都可能把名稱改回 `vultr`、重新產生 `/etc/hosts`；
沒寫進 `/etc/hosts` 的話，`sudo` 會報 `unable to resolve host`：

```bash
sudo hostnamectl set-hostname kawa-deb
echo 'preserve_hostname: true' | sudo tee /etc/cloud/cloud.cfg.d/99-kawa.cfg
# /etc/hosts 要改 cloud-init 的範本，直接改 /etc/hosts 重開機就沒了
echo '127.0.1.1 kawa-deb' | sudo tee -a /etc/cloud/templates/hosts.debian.tmpl
echo '127.0.1.1 kawa-deb' | sudo tee -a /etc/hosts
```

⚠️ **在 `cloud.cfg.d` 寫 `manage_etc_hosts: false` 沒有用**（2026-11-05 重開機後確認）：Vultr 的 vendor-data
（`/var/lib/cloud/instance/vendor-cloud-config.txt`）寫著 `manage_etc_hosts: true`，優先權比 `cloud.cfg.d` 高，
每次開機都會用範本重新產生 `/etc/hosts`。`preserve_hostname: true` 則有效，vendor-data 沒有設這一項。
同一份 vendor-data 還有 `ssh_pwauth: true`、`disable_root: false`（都被 `00-kawa.conf` 蓋掉），
以及一個 `linuxuser` 使用者。**Vultr 的 Debian 13 映像檔原本就有 `linuxuser`**（uid 1000、**在 sudo 群組**，
家目錄的建立時間比開機器還早），就算建機時沒勾 Limited User Login 也一樣。它的密碼是鎖住的、`authorized_keys` 是空的，
但這是一個用不到、卻有 sudo 權限的帳號，建好 kawa 後就刪掉：

```bash
sudo passwd -S linuxuser          # 第二欄是 L 代表密碼是鎖住的
sudo deluser --remove-home linuxuser
```

確認 kawa 能用金鑰登入、`sudo -v` 能過之後，**才**關閉密碼與 root 登入（順序反了會把自己鎖在外面，
只剩供應商的網頁 console 能救）：

```bash
sudo tee /etc/ssh/sshd_config.d/00-kawa.conf <<'EOT'
PermitRootLogin no
PasswordAuthentication no
EOT
sudo sshd -t && sudo systemctl reload ssh   # Debian 的 unit 叫 ssh，不是 sshd
sudo sshd -T | grep -Ei '^(permitrootlogin|passwordauthentication)'   # 兩個都要是 no
```

檔名一定要 `00-` 開頭：`sshd_config.d/` 照檔名順序讀，**同一個設定以第一次出現的值為準**。
cloud-init 放的 `50-cloud-init.conf` 可能寫著 `PasswordAuthentication yes`，取名 `99-` 會被它蓋過而且不會報錯。
最後要用 `sshd -T` 看實際生效的值，不要只看檔案內容。

### 2. 基本套件、IPv6、swap

```bash
sudo apt install -y rsync curl ca-certificates
```

- **IPv6**：`ip -6 route | grep '^default'` 沒有輸出，就照「主機 IPv6」節的 sysctl 整台關掉。repo 的 nginx
  已經沒有 `listen [::]`，新機不用管順序問題；**在裝 Docker 之前關**，就省掉一次 `systemctl restart docker`。
- **swap**（1 核 1G 必開，舊機曾因為完全沒 swap 被 OOM 殺掉 backend）。**先跑 `sudo swapon --show`**：
  Vultr 的 Debian 13 映像檔已經內建一個 2.3G 的 `/swapfile`，`/etc/fstab` 也寫好了。已經有 swap 的話，
  只要做 `swappiness` 那兩行；照下面整段跑會出現 `Text file busy`，還會在 fstab 多加一行重複的設定。
  `swapon` 在 `/usr/sbin`，一般使用者的 PATH 沒有這個目錄，要加 `sudo`。

  ```bash
  sudo fallocate -l 1G /swapfile && sudo chmod 600 /swapfile
  sudo mkswap /swapfile && sudo swapon /swapfile
  echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab   # 開機自動掛
  sudo sysctl -w vm.swappiness=10 && \
    echo 'vm.swappiness=10' | sudo tee /etc/sysctl.d/99-swappiness.conf
  free -h   # 驗證：SwapTotal 不能是 0
  ```

  `swappiness=10` 讓它只在真的吃緊時才換出，平時不拖慢反應。手動建的話 1G 就夠了（當初是因為舊機磁碟
  已經用了 14.8/23.4 GB）。新機 kawa-deb 沿用映像檔內建的 2.3G：23G 磁碟裝完只用了 5.2G，
  `/srv/kawa` 全部資料也只有大約 350MB（2026-11-04 實測），空間很充裕。

  > ⚠️ **舊機（first-laravel）從來沒做過這一步**，2026-08-09 實測 `SwapTotal: 0 kB`。
  > `docker-compose.yml` 的資源上限註解一度寫「1 核 1G（+2G swap）」，就是照本節假設了
  > 沒發生的事（已修正）。要確認實況一律跑 `free -h`，不要讀這份文件推論。

### 3. Docker（用官方 apt 來源，不用 Debian 自帶的 `docker.io`）

Debian 套件庫的 `docker.io` 版本較舊，compose plugin 也不同步。先到
https://docs.docker.com/engine/install/debian/ 確認支援清單有 trixie，再照官方步驟裝：

```bash
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc
sudo tee /etc/apt/sources.list.d/docker.sources <<EOT
Types: deb
URIs: https://download.docker.com/linux/debian
Suites: $(. /etc/os-release && echo "$VERSION_CODENAME")
Components: stable
Signed-By: /etc/apt/keyrings/docker.asc
EOT
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
sudo usermod -aG docker kawa                         # 重新登入後生效
docker compose version && systemctl is-enabled docker   # 必須是 enabled，開機才會自動起容器
```

Docker 的 apt 來源**不在** unattended-upgrades 的預設範圍內（預設只收 Debian 自己的來源：stable 與 Debian-Security），所以 daemon 只會手動升級。
這是刻意的：daemon 一升級，全部容器都會重啟。

### 4. fail2ban（原本只在舊機手動設定，2026-11 起記錄在這裡）

舊機的 `/etc/fail2ban/jail.local` 只有一個 `sshd` jail，用遞增式 ban。2026-07-02 才從 `bantime = 0`
（永久 ban）改過來，因為永久 ban 讓 `/var/lib/fail2ban/fail2ban.sqlite3` 長到 38M。下面的值照抄舊機
（2026-11-04 確認），只把 `logpath = /var/log/auth.log` 換成 `backend = systemd`：

```bash
sudo apt install -y fail2ban python3-systemd
sudo tee /etc/fail2ban/jail.local <<'EOT'
[sshd]
enabled  = true
port     = ssh
filter   = sshd[mode=aggressive]
backend  = systemd

maxretry = 3
findtime = 600
bantime  = 1h
bantime.increment = true
bantime.factor    = 24
bantime.maxtime   = 8w
EOT
sudo systemctl restart fail2ban
```

`backend = systemd` 是必要的：Debian 12 起預設不裝 rsyslog，沒有 `/var/log/auth.log`，用檔案 backend 的
sshd jail 會起不來，或是什麼都抓不到。`maxretry = 3`，所以下面的驗證只要失敗一次就好，
連試三次會 ban 到自己。被 ban 了就從供應商的網頁 console 執行 `fail2ban-client set sshd unbanip <IP>`。

**`mode=aggressive` 是必要的**（2026-11-04 新機上線當天發現）：關掉密碼登入之後，掃描器試 root 時沒機會送密碼，
log 只會留下 `Connection closed by authenticating user root ... [preauth]`。預設的 `mode=normal` 不算這種紀錄，
結果同一個 IP 每 1.5–2 分鐘試一次、試了一個多小時都沒被 ban。用今天的 log 實測：normal 只抓到 8 行，aggressive 抓到 46 行：

```bash
sudo fail2ban-regex systemd-journal "sshd[mode=normal]" | grep -A1 '^Lines'
sudo fail2ban-regex systemd-journal "sshd[mode=aggressive]" | grep -A1 '^Lines'
```

（`journalmatch` 不能從命令列傳進去，寫成 `systemd-journal[journalmatch=...]` 會報 `TypeError`。filter 本身已經定義了，直接用 `systemd-journal` 就好。）
副作用：自己連線時，在金鑰密碼的提示出現時按 Ctrl+C 中斷，也會被算一次失敗。

**驗證（這是靜默失敗型的設定，一定要做）**：從別台用不存在的帳號連一次（`ssh no-such-user@新機IP`），
`sudo fail2ban-client status sshd` 的 `Total failed` 要增加。2026-11-04 在 Debian 13 實測可以抓到：
OpenSSH 10 的認證 log 雖然改由 `sshd-session` 行程寫出，但它屬於 `ssh.service`，會被
`Journal matches: _SYSTEMD_UNIT=ssh.service + _COMM=sshd` 比對到（`+` 是「或」）。
如果數字不動，用 `sudo journalctl -u ssh -n 20` 看那次失敗的 log 有沒有出現。

### 5. 自動安全更新 + 自動重開機

```bash
sudo apt install -y unattended-upgrades
sudo dpkg-reconfigure -plow unattended-upgrades   # 選 Yes
sudo tee /etc/apt/apt.conf.d/52kawa-reboot <<'EOT'
Unattended-Upgrade::Automatic-Reboot "true";
Unattended-Upgrade::Automatic-Reboot-Time "20:00";
EOT
sudo unattended-upgrade --dry-run --debug 2>&1 | grep -i 'allowed origins'   # 要看到 Debian-Security
```

- `Automatic-Reboot-Time` 用的是**主機時區**，用 `timedatectl` 確認。主機是 UTC 的話，`20:00` 就是台灣凌晨 04:00。
- 只有裝了需要重開的更新（主要是核心）才會真的重開，通常每月 0–2 次。
- **重開會清空 valkey，全站會員和 admin 都會被登出**（見「邊界原則」的 valkey 段）。這是用來換核心修補的代價。

**看到新聞說 Debian 修了某個 CVE、想確認自己有沒有補上**，不用一則一則對，用 `debsecan` 一次列出
「Debian 已經修好、但這台還沒裝」的套件，沒有輸出就代表都補上了：

```bash
sudo apt install -y --no-install-recommends debsecan   # 一定要加，不然會順便裝郵件伺服器 exim4
debsecan --suite trixie --only-fixed --format packages
uname -r; cat /var/run/reboot-required 2>/dev/null     # 核心要重開機才會生效，另外確認
```

2026-11-06 第一次裝的時候沒加 `--no-install-recommends`，exim4 被當成建議套件一起裝了進來，
還設成開機自動啟動、監聽 `127.0.0.1:25`，事後才 purge 掉。

### 6. 驗收

```bash
grep PRETTY /etc/os-release; uname -r
free -h                                   # Swap 不是 0
ip -6 route                               # 關掉 IPv6 後應該沒有輸出
docker compose version; systemctl is-enabled docker
sudo fail2ban-client status sshd
systemctl list-timers apt-daily-upgrade.timer
```

## 全新機器 bootstrap

從零把整站架在一台新 VPS 上的流程。

### 0. 前提

- 新 VPS：照「新機 OS 準備」節做完（使用者 / SSH、IPv6、swap、Docker、fail2ban、自動更新）
- Cloudflare DNS：`kawa.homes`、`*.kawa.homes`、`www.kawa.homes` 三筆 A record（橘雲）指向新機 IP；
  SSL/TLS 模式 **Full (Strict)**。`api` / `axum` / `media` 沒有單獨的紀錄，都是透過 `*` 萬用紀錄連過來
  （2026-11-04 確認）；憑證是 `kawa.homes` + `*.kawa.homes` wildcard，新增子網域不用重簽，也不用另外加紀錄
- 若新機**取代**舊機：GitHub secrets 更新 `VULTR_HOST` / `VULTR_USERNAME` / `SSH_PRIVATE_KEY`

### 1. 建持久層

```bash
docker run --rm -v /srv:/srv alpine sh -c "mkdir -p /srv/kawa/env && chown -R $(id -u):$(id -g) /srv/kawa"
```

照 `env.example/kawa.env` 建 `/srv/kawa/env/kawa.env`。**新站是產新值不是抄舊值**：
`JWT_SECRET` 產新隨機字串、`POSTGRES_PASSWORD` 自訂（空庫，首次啟動用它初始化，
`DATABASE_URL` 內密碼要同步）、OAuth secrets 沿用既有 app。
`kawa.env` 與 `cloudflare.ini` 放好後都要 `chmod 600`（`kawa.env` 裡有 JWT_SECRET 與 DB 密碼；
舊機的 `kawa.env` 一直是 `664`，2026-11-04 搬家時才發現）。

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

⚠️ **`name` 必填、而且它才是登入帳號**：`users.name` 是 `NOT NULL` 且無預設（少了它整句 INSERT
就失敗），登入查的是 `WHERE name = $1`（`repositories/users.rs`，2026-07-06 起 email 降為選填）。
只塞 email 的話不但寫不進去，寫進去也登不了。

```bash
# 產 bcrypt hash
docker run --rm python:3-alpine sh -c "pip -q install bcrypt && python -c \"import bcrypt;print(bcrypt.hashpw(b'你的密碼', bcrypt.gensalt()).decode())\""

docker exec -it database psql -U kawa -d kawa -c "
  INSERT INTO users (name, email, password) VALUES ('admin', 'you@example.com', '<上面的hash>');
  INSERT INTO user_roles (user_id, role_id)
    SELECT u.id, r.id FROM users u, roles r
    WHERE u.name = 'admin' AND r.name = 'super_admin';
"
```

登入頁填的是上面那個 `name`（`admin`）＋密碼，不是 email。

### 5. 後台補 runtime 設定

登入 `/admin` → settings：OAuth client id / redirect URL、SMTP 帳密等
（存 `app_settings` 表，migration 有預設值，OAuth / SMTP 要填真值才能用）。

### 帶資料搬家的變體

不是空站而是搬家：跳過步驟 4–5，舊機停機後把整個 `/srv/kawa`
（env + uploads + torrents + dbdata）rsync 到新機同路徑；
憑證在新機重發（DNS-01 不依賴舊機）比搬 volume 簡單。
完整的搬家操作順序見下方「換 VPS 搬家 runbook」。

## 換 VPS 搬家 runbook（既有站台整機遷移）

把活著的站台從舊 VPS 搬到新 VPS。核心是 `/srv/kawa` 與 `deploy/` 的邊界已經切乾淨，
**產出值（JWT/密碼/OAuth）不重產，整個 `/srv/kawa` 原樣搬過去**。

### 前提：搞清楚哪些東西不用碰

- **Registrar（網域註冊商，Google Domains → Squarespace）不參與。** DNS 權威在 Cloudflare
  （`dig NS kawa.homes` → `*.ns.cloudflare.com`），只需確認 Squarespace 那邊 NS 委派沒被改回預設。
- **對外沒有 DNS 傳播延遲。** 網域是 proxied（橘雲）狀態，對外 A record 永遠是 Cloudflare 邊緣 IP
  （`104.21.x` / `172.67.x`），不是 VPS 真實 IP。換機只改 Cloudflare 後台那筆「回源 IP」，
  對外紀錄不變 → 近乎即時生效、不用等 TTL。
- **PG 同版本（18）不需 dump/restore。** 直接搬 `dbdata/` 目錄即可（跨大版本才要走 PG 升級 runbook）。

### 0. 新機前置（站台不中斷）

舊站照常運作，可以分幾天做。**只要 `dbdata` 還沒搬，就不要在新機跑 `docker compose up`**：
PG 會在空的 `/srv/kawa/dbdata` 建一個新的空資料庫，backend 也會在上面跑 migration。

- 照「新機 OS 準備」節做完（使用者 / SSH、IPv6、swap、Docker、fail2ban、自動更新），第 6 步驗收要全過
- 建持久層根目錄：`sudo install -d -o kawa -g kawa /srv/kawa`
- 舊機的 kawa 在 `www-data`（gid 33）群組，新機比照：`sudo usermod -aG www-data kawa`

```bash
# 本機：部署設定先同步過去
rsync -av deploy/ kawa@新VPS:~/kawa-deploy/

# 新VPS：驗證 compose、先拉 image、先申請憑證（DNS-01 只起 certbot 一個容器，不依賴流量在哪台）
cd ~/kawa-deploy
docker compose config --quiet && docker compose pull
bash shells/issue-cert.sh
```

接著準備搬資料。⚠️ **`/srv/kawa` 不能用一般使用者 rsync**：`dbdata/` 的擁有者是 postgres 容器的 uid 70、
權限 `0700`，kawa 讀不到。兩端都要以 root 執行，並加 `--numeric-ids` 原樣保留數字 uid（新機不需要有同名帳號）。
做法是**在舊機以 root 推到新機**，新機那端透過 kawa 的 SSH，加上暫時開放的免密碼 `sudo rsync`：

```bash
# 新VPS：暫時允許 kawa 免密碼 sudo rsync（搬完就刪）
echo 'kawa ALL=(root) NOPASSWD: /usr/bin/rsync' | sudo tee /etc/sudoers.d/kawa-rsync

# 舊VPS（kawa）：產一把金鑰，公鑰加進新機 kawa 的 authorized_keys
ssh-keygen -t ed25519 -f ~/.ssh/id_ed25519 -N ''

# 舊VPS（root，su -）：測試整條路徑，要印出 rsync 版本
ssh -i /home/kawa/.ssh/id_ed25519 kawa@新VPS 'sudo -n rsync --version | head -1'

# 舊VPS（root）：預熱 dbdata 以外的檔案（可重複跑）
rsync -a --numeric-ids --exclude dbdata/ --info=progress2 \
  -e "ssh -i /home/kawa/.ssh/id_ed25519" --rsync-path="sudo rsync" \
  /srv/kawa/ kawa@新VPS:/srv/kawa/
```

2026-11-04 實測資料量：`dbdata` 253M、`uploads` 95M、`torrents` 幾乎是空的，預熱幾秒就結束。
`kawa.env` 原本是 `664`，裡面有秘密值，兩台都改成 `chmod 600`（compose 以 kawa 身分讀，不受影響）。

### 1. 停站、搬資料（停機開始）

```bash
# 舊VPS：停站
cd /home/kawa/kawa-deploy && docker compose down

# 舊VPS（root）：推完整 /srv/kawa（含 dbdata；--delete 對齊，增量只補差異）
rsync -a --numeric-ids --delete --info=progress2 \
  -e "ssh -i /home/kawa/.ssh/id_ed25519" --rsync-path="sudo rsync" \
  /srv/kawa/ kawa@新VPS:/srv/kawa/
```

### 2. 新機起服務

```bash
# 新VPS
cd ~/kawa-deploy && docker compose up -d && docker compose ps
sudo rm /etc/sudoers.d/kawa-rsync     # 撤掉暫時權限
```

憑證在第 0 步就申請好了，nginx 會直接起來。

### 3. 切流量到新機（Cloudflare，非 registrar）

- Cloudflare dashboard → DNS → Records，把 `kawa.homes` / `*.kawa.homes` / `www.kawa.homes` 三筆 A record 的
  **回源 IP** 從舊機改成新機 IP（`api` / `axum` / `media` 都是透過 `*` 連過來，沒有單獨的紀錄）。
  橘雲維持開啟，SSL/TLS 維持 **Full (Strict)**。`_domainconnect` 那筆 CNAME 是 Google Domains 留下來的，不用動。
- 2026-11-04 搬家：舊機 first-laravel（Ubuntu 20.04）`64.176.40.38` → 新機 kawa-deb（Debian 13，Tokyo）`45.76.102.67`。
  舊機 2026-11-05 退租（Vultr Destroy，沒有留 snapshot）。退租前盤點過：沒有自訂 cron 或服務，
  `/srv/kawa` 以外只剩 docker-env 時代的腳本、已封存的 docker-env repo（沒有未 push 的修改），
  以及 2023 年的 Laravel 專案（不需要保留）。
- **GitHub secrets** 更新成新機，否則下次 push 會 deploy 到舊機：
  `VULTR_HOST` / `VULTR_USERNAME` / `SSH_PRIVATE_KEY`（deploy.yml / backend.yml / frontend.yml 三條 CI 共用）。
  CI 是用 **kawa** 登入（2026-11-04 確認：舊機 root 沒有 `authorized_keys`），所以只要新機 kawa 的
  `authorized_keys` 裡有同一把公鑰（就是建機時上傳到 Vultr 的那把），通常只需要改 `VULTR_HOST`。

### 4. 驗證與善後

```bash
# 新VPS
docker compose ps                     # 六個服務 Up、database healthy
curl -sI https://kawa.homes | head -1
curl -s https://api.kawa.homes/health   # {"status":"ok"}（根路徑無路由、回 404 是正常的，別拿 / 當探針）
# 瀏覽器：前台登入（驗 JWT_SECRET 搬對）、後台登入、WS 頁面、圖片/torrent 讀取
```

- 從新機 kawa 的 `authorized_keys` 移除搬資料用的舊機金鑰（`sed -i '/kawa@<舊機hostname>/d' ~/.ssh/authorized_keys`）。
  回滾用不到這把金鑰，留著的話，舊機一旦被入侵就能直接登入新機。
- 新機 `kawa.env` 確認是 `600`（`ls -l /srv/kawa/env/kawa.env`；舊機搬過來時是 `664`，`rsync -a` 會把權限原樣帶過來）。
- 站台穩定幾天後才關舊機、退租。
- **回滾**：Cloudflare 回源 IP 指回舊機 + GitHub secrets 還原 + 舊機 `compose up -d`（舊機資料還在）。

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
curl -s https://api.kawa.homes/health   # {"status":"ok"}（根路徑無路由、回 404 是正常的，別拿 / 當探針）
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
curl -s https://api.kawa.homes/health   # {"status":"ok"}（根路徑無路由、回 404 是正常的，別拿 / 當探針）
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
