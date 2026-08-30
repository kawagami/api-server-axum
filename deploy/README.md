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
  後台 / 前台使用者都一樣。
- 其餘 key 掉了無感：權限快取（1h）、oauth state（5m）、ws ticket（30s）都會自動重算。

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

- 改 `deploy/**` → `deploy.yml`：scp 到 staging → `compose config` 驗證 → rsync 覆蓋 `~/kawa-deploy` → `compose up -d` → 一次性容器 `nginx -t` → `--force-recreate` nginx。
- 改 `backend/**` / `frontend/**` → 各自 workflow build image 後 SSH：`cd ~/kawa-deploy && docker pull … && docker compose up -d` → **`nginx -s reload`**（原因見下）。
- 三條 deploy 共用 `concurrency: vps-deploy`，序列化不撞車。

### ⚠ 換 image 後必須 reload nginx —— upstream IP 會過期

`proxy_pass http://backend:3000` 是**靜態 hostname**，nginx 只在載入設定時解析一次、之後永久
快取那個 IP。`docker compose up -d` recreate 掉 backend / frontend 容器後 Docker 可能配到新 IP，
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

**結論：瓶頸不在 origin（只佔 3%），而是 CF 免費方案把本站的網域丟到新加坡而非台灣節點。** 腳本會自動比對本站與 `cloudflare.com` 的 `cf-ray` 節點代碼並標出差異 —— 那個 baseline 代表「這條線路本來碰得到的最近節點」。

## blog 搜尋限流（`search` zone，2026-08-27）

`?q=` 的搜尋是全站最貴的公開查詢：列表端點的 `count` 那一半無法靠 LIMIT 提早結束，一個命中大量文章的常見詞 ≈ **60ms 的 PG CPU**（2026-08-27 實測，2000 篇 / 3MB 表）。general 的 30r/s 放進來就是單核吃滿。

`nginx.conf` 的 `search` zone（20r/m，burst 5）用一個 map 當 key：**沒有 `?q=` 時 key 是空字串，limit_req 對空 key 不計數**，所以同一條指令只對「帶 `q=` 的請求」生效，一般瀏覽不計數。

### ⚠ map 只看 `q=`、不看路徑 —— 會誤中的路徑要開獨立 location（2026-08-30）

`$arg_q` 是**任何**帶 `q=` 的網址，與「是不是搜尋」無關。Next.js 的圖片最佳化網址
`/_next/image?url=…&w=1920&q=75` 那個 `q=` 是**畫質**，照樣被算進 search zone —— 圖多的 blog
一次併發十幾張，第 7 張起（burst 5 + 1 個 rate token）一律 429，在瀏覽器上就是
**「原圖網址打得開、頁面上卻破圖」**（2026-08-30 實測確認，commit 879a22d 修）。

修法是把會誤中的路徑拉出獨立 location、只掛 `general`：目前已這樣處理的是
`conf.d/kawa.homes.conf` 的 `location /_next/`（image 最佳化 + static 資產全段）。
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
> **必須 recreate nginx，不能只 reload** —— `limit_req_zone search` / `proxy_cache_path`
> 都在 `nginx.conf` 裡，只 reload 的話容器吃到的是舊檔，會報 `zero size shared memory
> zone "search"`（看起來像語法錯，其實是設定檔沒更新）。理由見下一節。
> CI 的 `deploy.yml` 本來就會 `--force-recreate` nginx，走 CI 不必額外處理。

### ⚠ nginx.conf 是單檔 bind mount —— 改它必須 recreate 容器

`docker-compose.yml` 把 `./nginx/nginx.conf` 以**單一檔案**掛進容器，而單檔 bind mount 綁的是 **inode**。rsync 的預設行為是「寫暫存檔再 rename」，會換掉 inode，**執行中的容器因此看不到新的 `nginx.conf`**。相對地 `./nginx/conf.d` 是**目錄**掛載，目錄內的檔案變動會正常反映。

後果是新舊混用，而且錯誤訊息會指向錯的地方 —— 例如在 `conf.d` 新增 `limit_req zone=auth`、同時在 `nginx.conf` 新增對應的 `limit_req_zone`，容器只吃到前者，報的卻是 `zero size shared memory zone "auth"`，看起來像語法錯，實際是設定檔沒更新（2026-07-25 實際踩過）。

因此 `deploy.yml` 的做法是：

1. **用一次性容器驗證**，掛載當下磁碟上的檔案（不是執行中容器裡那份舊的）。借 `--network container:nginx` 取得 compose 網路，否則 `proxy_pass http://backend:3000` 在載入階段就 host not found。
2. 通過後 **`docker compose up -d --force-recreate --no-deps nginx`** —— 只有 recreate 會重新解析 bind mount，`restart` 與 `nginx -s reload` 都不會。

手動改 VPS 上的 nginx 設定時同理：動到 `nginx.conf` 就得 recreate，只動 `conf.d` 才能 reload 了事。

## 主機 IPv6：必須整台關掉（2026-08-09）

這台 VPS（以及任何同型的新機）**沒有可用的 IPv6，但 IPv6 stack 是開著的**：

```
ip -6 addr    → 只有 link-local fe80::，沒有任何全域位址
ip -6 route   → 只有 ::1 與 fe80::/64，沒有 ::/0 預設路由
```

於是 glibc 與 Go 都判定「這台有 IPv6」，對外連線先試 AAAA 再撞牆。實際災情（14 天內）：

| 時間 (UTC) | 誰 | 目的地 | 錯誤 |
|---|---|---|---|
| 08-05 23:30 | lettre | smtp.gmail.com | `EADDRNOTAVAIL` 政府標案通知信沒寄出 |
| 08-06 00:57 | lettre | smtp.gmail.com | 同上，torrent 完成通知信沒寄出 |
| 08-09 07:21 / 07:53 | reqwest | www.googleapis.com | 同上，member 用 Google 登入回 502 |
| 08-09（手動） | **docker daemon** | auth.docker.io | `ENETUNREACH`，`docker pull` 失敗 |

最後一列最關鍵：那是**宿主機自己**，證明問題不在任何容器裡。而部署腳本第一行就是
`docker pull kawagami77/api-server:latest`（配 `set -e`），所以這也會讓部署紅燈。

> ⚠️ **上表中間兩列的歸因後來被推翻。** 宿主機的 IPv6 已於 08-09 08:46 (UTC) 關閉，
> 而 backend 的同型 connect 失敗在 16:14 又出現了一次 —— 關掉宿主機 IPv6 修好的是
> **dockerd**（Go，啟動時探測一次）與 lettre，容器內 glibc 的 getaddrinfo 照樣回 AAAA。
> reqwest 那兩列的真正成因見下面「對外連線偶發 connect 失敗」。這一節其餘內容仍然成立：
> 宿主機 IPv6 該關，順序不可顛倒。

> ⚠️ **最後一列（dockerd / `docker pull`）的歸因也在 08-12 被推翻。** 同一個錯誤在
> 16:11 (UTC) 讓部署紅燈，而當時宿主機 IPv6 仍是關著的、且 07-27 以來沒重開機過：
>
> ```
> ip -6 addr show scope global   → 空
> ip -6 route show default       → 空
> sysctl net.ipv6.conf.{all,default}.disable_ipv6 → 均為 1（drop-in 檔完好）
> getent ahostsv6 auth.docker.io → 空
> journalctl -u docker           → dial tcp [2606:4700:4403::ac40:904e]:443:
>                                   connect: cannot assign requested address
> ```
>
> `getent` 空而 dockerd 拿得到 AAAA，是**兩條解析路徑**的差異：glibc `getaddrinfo` 套用
> **`AI_ADDRCONFIG`**（本機沒 v6 位址就濾掉 AAAA），**Go 的 resolver 不套**。
> ⇒ 這台上 `docker pull` **每次都有一腿撞 v6 失敗**，平時靠 v4 那腿救回來；關 IPv6 只會把
> errno 從 `EADDRNOTAVAIL` 換成 `ENETUNREACH`，AAAA 不會從 Go 眼前消失。
> **這條沒有乾淨的根治手段**，處理方式是部署腳本重試（見下面那節）。

### 修法與**不可顛倒的順序**

1. **先**把 `nginx/conf.d/*.conf` 的 `listen [::]` 全部拆掉並部署
   （已於 2026-08-09 完成）。少了這步，第 2 步之後 nginx 會
   `socket() [::]:80 failed (97: Address family not supported by protocol)` 起不來，**全站掛掉**。
2. **再**在宿主機關閉 IPv6：

```bash
sudo tee /etc/sysctl.d/99-disable-ipv6.conf <<'EOF'
net.ipv6.conf.all.disable_ipv6 = 1
net.ipv6.conf.default.disable_ipv6 = 1
EOF
sudo sysctl --system
sudo systemctl restart docker   # daemon 的 IPv6 能力是行程啟動時探測一次，不重啟不會生效
```

> `systemctl restart docker` 會重啟所有容器，比照一次完整部署安排時間。

### 排查時的兩個坑（都踩過）

- **容器層級的 `sysctls: net.ipv6.conf.all.disable_ipv6=1` 解決不了。** 2026-08-09 試過，容器
  帶著它在 07:51:34 重建，07:53:37 同一個錯誤照樣發生、訊息逐字相同。那兩行留在 compose 裡
  只當衛生，不是解法。
- **不要用 alpine 測 DNS 行為。** alpine 是 musl，沒有 IPv6 位址時會自己濾掉 AAAA；
  backend 的 distroless/cc 是 **glibc**，不帶 `AI_ADDRCONFIG` 就照回 AAAA。
  要測就用 `debian:bookworm-slim`，並且 `--network container:backend` 共用同一個 netns 與 DNS。

### 應用層的防護（不取代上面）

`utils/reqwest.rs::send_retrying` 與 `services/email.rs` 對**連線階段**的失敗重試。
兩邊的預算**已經不一樣了**（08-12 起，別再寫成同一組數字）：reqwest **4 次 / 指數
250-500-1000ms**（合計 1.75 秒，理由見「三種失敗的共同根因與修法」）、email 仍是
**3 次 / 線性 300-600ms**。刻意只重試「請求還沒送達對方」的失敗 —— 對方已回狀態碼、
或信已進 SMTP 對話的一律不重試，免得同一封中獎通知寄兩次。

對 reqwest 那條路徑而言這不只是防護，而是**對症的解**，理由見下一節。

## 對外連線偶發 connect 失敗：Docker 內嵌 DNS 冷快取（2026-08-10）

症狀：**backend 容器重啟後的第一個對外請求**偶爾在 connect 階段失敗，
`Cannot assign requested address (os error 99)`，重試一次就通。實際觀測（UTC）：

| backend-ci 完成（≒容器重啟） | 失敗時間 | 間隔 |
|---|---|---|
| 08-09 06:49:08 | 07:21:31 | 32 分 |
| 08-09 07:51:36 | 07:53:37 | **2 分** |
| 08-09 16:12:24 | 16:14:11 | **2 分** |

最後一筆在宿主機關閉 IPv6（08:46）之後 —— 所以這不是上一節那個問題。

### 推導

錯誤能浮出水面，代表**解析結果裡只有 AAAA、一筆 A 都沒有**。依據是
hyper-util 0.1.20 `client/legacy/connect/http.rs`：

```rust
// ConnectingTcp::new —— 只有一種家族時不建 fallback
if fallback_addrs.is_empty() {
    return ConnectingTcp { preferred, fallback: None };
}

// ConnectingTcp::connect —— 有 fallback 時，preferred 一報錯就立刻換另一邊
if result.is_err() { future.await } else { result }
```

也就是說 A 與 AAAA 同時存在時，v6 撞 `EADDRNOTAVAIL` 會**靜靜地**退回 v4 接上，
呼叫端什麼都看不到（happy eyeballs 的 300ms 延遲只影響誰先開始，不影響失敗後的回退）。
會硬失敗只剩「fallback 是空的」這一種可能。

AAAA-only 的答案來自 **Docker 內嵌 DNS（127.0.0.11）**：它把 A 與 AAAA 拆成兩個上游查詢，
掉一個就只回另一個。快取是 per-network 的，**容器剛重啟時是冷的**，第一次查最容易掉；
查成功之後兩筆都進快取，後面就穩了。這解釋了為什麼失敗總是單發、總是在部署後不久。

### 結論：不用再修（**這個標題在 08-12 過期了**）

`send_retrying` 重試時會**重新 resolve**，第二次拿到 A 就接上 —— 正好對症。
08-09 16:14 那次實測即是如此：`logs` 表只有 1 筆 WARN、沒有 ERROR，使用者拿到 200。

⚠️ 「不用再修」只對**這一節那個成因**成立。08-12 之後仍動了三處（重試預算、`dns_opt`、
部署腳本 `docker pull` 重試），因為抖動的**代價**與**可見度**還有問題 —— 見
「三種失敗的共同根因與修法」。讀到這個標題不要當成「這條線已經結案」。

⚠️ **不要改成 `reqwest` 的 `local_address(0.0.0.0)`「只走 IPv4」。** 那確實會讓
`split_by_preference` 濾掉所有 AAAA，但 AAAA-only 的答案在濾完之後是**空清單**，
hyper 照樣回 `tcp connect error`。治不了這個症，還永久關掉 IPv6 的可能性。

### 判讀提示

`utils/reqwest.rs` 的重試 WARN 已經會印整條 source chain（`error_chain()`），
所以下次再抖，`logs` 表那行就直接看得到 errno；在那之前只印最外層的
`error sending request for url (…)`，什麼都推不出來。

## 對外連線第三種：解析整個失敗（`EAI_NONAME`，2026-08-11）

與上一節**不同的錯誤**，別套上一節的結論：

```
dns error
  <- failed to lookup address information: Name or service not known
```

上一節是「解析回了 AAAA、接不上」；這個是 **A/AAAA 一筆都沒拿到**，連線根本沒開始。

觀測（UTC，全部打 `www.twse.com.tw`，全部單發）：

| 時間 | 誰 | 後果 |
|---|---|---|
| 08-03 20:00:30 | `FetchStockDayAll` | WARN attempt 1/3，1 小時後第 2 次成功 |
| 08-05 20:00:00 | 同上 | 同上 |
| 08-10 20:00:00 | 同上 | 同上 |
| 08-11 00:00:00 | `ConsumePendingStockChange`（stock_no=6257） | ERROR，該筆留在 pending，下一分鐘重跑 |

**不是冷快取**：backend 最後一次重建是 08-09 16:40 UTC，上表後兩筆距它 27～31 小時。
（08-10 16:06–17:20 那六次 CI 是 frontend-ci，不動 backend 容器。）
同期 `googleapis` / `gmail` / 採購網 / 台彩**零筆** —— 只有 twse 這個網域中。

成因未定，可能是上游 DNS 或內嵌 DNS 轉發的抖動。**沒有再往下追**，理由是形狀與上一節一致
（單發、重新 resolve 就通），而 08-11 已把 `send_retrying` 補到 `get_raw_html_string` /
`get_json_data`（＝ TWSE 全部路徑）與 `services/lotto.rs`、`services/gov_tenders.rs`，
現在第一次抖就在 250ms 後重解析（08-12 前是 200ms），而不是等 job 層退避 3600 秒。

⚠️ **若之後 `logs` 表開始出現「重試到最後一次」的 WARN**，代表重試已經吃不下，
那時才值得往宿主機 `/etc/resolv.conf` 與內嵌 DNS 的上游查。~~目前只出現過第 1 次。~~
**08-12 兌現了**（當時的訊息是「第 2/3 次」共 4 筆），處理見下一節。
⚠️ 預算已於 08-12 改成 4 次，**現在要盯的字串是「第 3/4 次」** —— 舊的「第 2/3 次」
不會再出現，拿它當監控條件會永遠是零。

## 三種失敗的共同根因與修法（2026-08-12）

上面三節是三種 errno，但 08-12 把 08-05 起的紀錄攤開看，形狀是同一件事。

### 統計

`scripts/kawa-logs logs -q "對外請求暫時性失敗" --from 2026-08-05`：

| | 筆數 |
|---|---|
| 重試 WARN | 9（≒5 次抖動事件，每次記 2 筆） |
| job 層 `服務連接失敗` | 5，**全部下一輪重試成功** |
| ERROR（24h） | 0 |

errno 分佈：`os error 99` ×4、`Name or service not known` ×2、
`No address associated with hostname` ×2。網域分散（台彩 / twse / 採購網 / googleapis），
**不再只有 twse**，所以上一節「只有 twse 中」那句已過期。

### 關鍵觀察：全部落在 cron tick 的 `:00.x`

`23:00:00.3` / `20:00:00.4` / `17:00:00.8` / `07:00:00` / `08:00:00`。同一秒觸發的不只一支
job（每分鐘的 `CollectSystemMetrics` / `ConsumePendingStockChange` /
`FetchHistoricalClosingPrices` 三支，加上該點的日排程），**並行的 getaddrinfo 撞在一起**。

這把三種 errno 收成一個根因：內嵌 DNS 把一次解析拆成 A 與 AAAA 兩個上游查詢，
掉一個就只回另一個（→ AAAA-only，`os error 99`）、都掉就整個失敗（→ `EAI_NONAME`）、
回了但沒有可用 family（→ `EAI_NODATA`）。冷快取只是「最容易掉的時機」之一，不是唯一。

### 一條看起來像根因、但推不動的線索：內嵌 DNS 的 v6 上游（08-13）

`journalctl -u docker` 每建一個容器就印一次這兩行：

```
No non-localhost DNS nameservers are left in resolv.conf.
  Using default external servers: [nameserver 8.8.8.8 nameserver 8.8.4.4]
IPv6 enabled; Adding default IPv6 external servers:
  [nameserver 2001:4860:4860::8888 nameserver 2001:4860:4860::8844]
```

宿主機 `/etc/resolv.conf` 只有 localhost（systemd-resolved），所以 dockerd 退回內建預設，
**而它連 IPv6 那兩台一起加**。這台沒有可用 IPv6 → 四台上游有兩台永遠打不通。

⚠️ **一度把這寫成「每次轉發都有一半機率挑到死的上游」＝根因。那個推論不成立**（08-13 自我推翻，
留在這裡免得下一個人重走）：看 log 的**順序** —— v4 兩台是 `Using default external servers`，
v6 兩台是後面 `Adding default IPv6 external servers`，**append 在清單尾端**。libnetwork 的
resolver 是依序轉發、成功就停，所以那兩台 v6 只有在 8.8.8.8 與 8.8.4.4 都失敗時才會被碰到
—— 是永遠用不到的 fallback，不是每次都在抽的籤。

⇒ 「有兩台死上游」是**事實**（log 原文在上，值得知道），但它**解釋不了**那些失敗。
真正還沒解釋的仍是「內嵌 DNS 對 127.0.0.11 那一問為什麼會掉」。
對應的候選修法見下面「候選修法：查完認定沒用」。

### 兩處改動

1. **`docker-compose.yml` backend 加 `dns_opt: [single-request-reopen, timeout:2, attempts:3]`**
   —— glibc 預設把 A/AAAA 塞同一個 UDP socket 平行送，改成序列 + 換 socket。
   ⚠️ 序列化只在**單一 getaddrinfo 內部**，不同連線的解析照樣並行、也不佔 async worker
   （hyper 的 `GaiResolver` 走 tokio blocking pool）；成本是建新連線時多一個 RTT，
   對象是同 netns 的 127.0.0.11，而各連線池讓穩態幾乎不再解析。**對承載量無影響。**
   `timeout:2 attempts:3` 比 glibc 預設（5 秒 ×2 輪）更快放棄，是降延遲方向。
   只對 glibc 有效 —— 基底 distroless/cc-debian12 成立。
2. **`utils/reqwest.rs::send_retrying` 的重試預算 3 次 / 600ms → 4 次 / 1.75 秒**（指數 250/500/1000ms）
   —— 原本三次全在 600ms 內用完，抖動撐過 600ms 就整批失敗、掉到 job 層退避
   **1800 秒（gov_tenders）／ 3600 秒（TWSE）**。08-11 23:00 那筆正是如此（1.1 秒用完三次）。
   多等 1.75 秒換掉半小時。
   ⚠️ **這裡原本寫「呼叫端全是排程 job、沒有使用者在等」，是錯的**（08-13 修正）：
   `services/oauth.rs` 有 5 支走 `send_retrying`，全在 member 登入路徑上。抖動時退避疊加
   （Google 序列 2 支 → 最壞多 3.5 秒、GitHub 3 支 → 5.25 秒），仍在 60 秒 request timeout
   內，但這個交換**不是零代價**。要再放寬預算前先確認登入路徑吃得下。

3. **部署腳本 `docker pull` 重試 3 次**（08-13，`backend.yml` / `frontend.yml` 各一處）——
   dockerd 那腿 v6 失敗沒有根治手段（見「主機 IPv6」節的第二個 ⚠️），裸的一發 + `set -e`
   等於把一次 DNS 抖動變成部署紅燈。退避 5 / 10 秒，三次都失敗才紅燈。
   `[ "$i" = 3 ] && exit 1` 在 `set -e` 下的行為已用 bash 與 dash 各實測過（0/1/2 次失敗
   都會繼續往下走，3 次才 exit 1）。
   **這一項的證據最硬**：`6830e11` 那批 CI（16:08:35Z）裡 `frontend-ci` 的 `docker pull`
   在同一台機器、同一分鐘**成功**，`backend-ci` 的**失敗** —— 同時、同主機、不同結果，
   即「每次撥號的硬幣」，重試對症。

### 候選修法：查完認定沒用（08-13）

**`dns: [8.8.8.8, 1.1.1.1]`**（給 backend 指定 v4-only 上游）。寫好了又**撤掉**，因為
它要修的那個機制上一節已經自我推翻 —— v6 上游 append 在清單尾端、正常路徑碰不到，
拿掉它們不會改變任何事。**不要因為「反正無害」再把它加回來**：無害不是理由，
它會讓下一個人以為死上游那條線索已經處理過了。

若之後真要指定 `dns:`，先知道兩件事：`dns:` 設定的是內嵌 DNS **往外轉發**用的伺服器，
**不會繞過 127.0.0.11**（user-defined network 裡容器的 `resolv.conf` 永遠指向內嵌 DNS，
`database` / `valkey` 仍由它回答）；而且它與 `dns_opt` 打同一個症狀，一起上線就分不出哪個有效。

**還沒做、也還有價值的是量測** —— 開一次性 glibc 容器接同一個網路，壓一輪並發解析看
能不能重現（唯讀、不動任何容器）：

```bash
docker network ls          # 先確認網路名
docker run --rm --network kawa_default debian:12-slim sh -c '
  i=0; while [ $i -lt 40 ]; do
    for h in www.twse.com.tw api.taiwanlottery.com pcc-api.openfun.app oauth2.googleapis.com; do
      getent hosts $h >/dev/null || echo "FAIL $h"
    done &
    i=$((i+1))
  done; wait; echo done'
```

⚠️ **不要用 `docker exec valkey` 代替**：alpine 是 musl，resolver 行為與 backend 的 glibc 不同，
量到的結果不能代表 backend。

**要 A/B 就同一輪跑兩次**：上面那個 `docker run` **不帶 `dns_opt`**（那是設在 backend service
上的，per-container），所以它量到的是**沒有** `single-request-reopen` 的對照組。加上
`--dns-opt single-request-reopen --dns-opt timeout:2 --dns-opt attempts:3` 再跑一次，
兩邊 FAIL 筆數的差就是 `dns_opt` 的實際效果 —— 不必等一兩週看 `logs` 表筆數。

### 還沒做（下一階段）

`reqwest` 的 `hickory-dns` feature：純 Rust resolver，自帶快取、不吃 glibc 那套行為，
DNS 流量比現在更少。先觀察前面那些的效果一兩週再決定。
⚠️ 換之前要確認容器內名字（`database` / `valkey`）的解析仍走 127.0.0.11。

## 全新機器 bootstrap

從零把整站架在一台新 VPS 上的流程。

### 0. 前提

- 新 VPS：建使用者 + SSH 金鑰、裝 docker（含 compose plugin）與 rsync、使用者加入 `docker` 群組
- **檢查 IPv6**：`ip -6 route | grep '^default'` 沒東西就照上一節整台關掉，否則對外連線
  （OAuth / SMTP / `docker pull`）會間歇失敗
- Cloudflare DNS：`kawa.homes`、`*.kawa.homes`、以及各子網域單獨那幾筆（`api` / `axum` / `media`）
  指向新機 IP；SSL/TLS 模式 **Full (Strict)**（憑證是 `kawa.homes` + `*.kawa.homes` wildcard，
  新增子網域不用重簽，但橘雲要單獨開一筆 record）
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

- 建使用者 + SSH 金鑰、裝 `docker`（含 compose plugin）+ `rsync`、使用者加入 `docker` 群組
- **開 swap**（1 核 1G 尤其必要——曾整機無 swap 被 OOM 殺 backend）：
  ```bash
  sudo fallocate -l 1G /swapfile && sudo chmod 600 /swapfile
  sudo mkswap /swapfile && sudo swapon /swapfile
  echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab   # 開機自動掛
  sudo sysctl -w vm.swappiness=10 && \
    echo 'vm.swappiness=10' | sudo tee /etc/sysctl.d/99-swappiness.conf
  free -h   # 驗證：SwapTotal 不能是 0
  ```
  `swappiness=10` 讓它只在真的緊繃時才換出，平時不拖慢反應。尺寸從 2G 改成 1G：
  現行這台磁碟 14.8/23.4 GB 且 torrent 還在寫，2G 太貪。

  > ⚠️ **現行這台（first-laravel）從來沒做過這一步**，2026-08-09 實測 `SwapTotal: 0 kB`。
  > `docker-compose.yml` 的資源上限註解一度寫「1 核 1G（+2G swap）」就是照著本節假設了
  > 沒發生的事（已修正）。**這是換機 runbook 的步驟，不代表現行機器的狀態** ——
  > 要確認實況一律跑 `free -h`，不要讀這份文件推論。
- 建持久層根目錄（kawa 無 sudo 密碼也行，用容器以 root 建）：
  ```bash
  docker run --rm -v /srv:/srv alpine sh -c "mkdir -p /srv/kawa && chown -R $(id -u):$(id -g) /srv/kawa"
  ```

### 1. 資料搬遷（停機開始，抓最小窗口）

`dbdata` 必須停機後搬才一致；uploads / torrents / env 可先做一次預熱 rsync，停機後再補增量。
rsync 兩端不能同時遠端，以下指令**在新機上跑、從舊機拉**（新機需能 SSH 到舊機；
或反過來在舊機 push，擇一）。

```bash
# 新VPS：（選配）停機前先預熱體積大的靜態檔，縮短停機窗口
rsync -av 舊VPS:/srv/kawa/uploads/   /srv/kawa/uploads/
rsync -av 舊VPS:/srv/kawa/torrents/  /srv/kawa/torrents/

# 舊機停站
ssh 舊VPS 'cd ~/kawa-deploy && docker compose down'

# 新VPS：停機後拉完整 /srv/kawa（含 dbdata 與 env；--delete 對齊，增量只補差異）
rsync -av --delete 舊VPS:/srv/kawa/  /srv/kawa/
```

### 2. 部署設定、起服務

```bash
# 本機
rsync -av deploy/ 新VPS:~/kawa-deploy/
# 新VPS
cd ~/kawa-deploy && docker compose config --quiet && docker compose up -d
```

nginx 此時起不來是正常的（新機還沒憑證）。

### 3. 新機重發憑證（DNS-01，不依賴舊機）

```bash
# 新VPS
cd ~/kawa-deploy && bash shells/issue-cert.sh && docker compose restart nginx
```

### 4. 切流量到新機（Cloudflare，非 registrar）

- Cloudflare dashboard → DNS → 把 `kawa.homes` / `*.kawa.homes`（以及 `api` / `axum` / `media`
  等單獨列出的那幾筆）的**回源 IP**
  從舊機改成新機 IP。橘雲維持開啟，SSL/TLS 維持 **Full (Strict)**。
- **GitHub secrets** 更新成新機，否則下次 push 會 deploy 到舊機：
  `VULTR_HOST` / `VULTR_USERNAME` / `SSH_PRIVATE_KEY`（deploy.yml / backend.yml / frontend.yml 三條 CI 共用）。

### 5. 驗證與善後

```bash
# 新VPS
docker compose ps                     # 六個服務 Up、database healthy
curl -sI https://kawa.homes | head -1
curl -s https://api.kawa.homes/health   # {"status":"ok"}（根路徑無路由、回 404 是正常的，別拿 / 當探針）
# 瀏覽器：前台登入（驗 JWT_SECRET 搬對）、後台登入、WS 頁面、圖片/torrent 讀取
```

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
