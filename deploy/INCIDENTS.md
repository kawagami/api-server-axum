# deploy — 事故紀錄與推導

主機網路（IPv6 / Docker 內嵌 DNS / 對外連線抖動）2026-08 的完整排查過程，**原樣**從 `README.md` 搬來。
**現況與操作規則以 `README.md` 的「主機 IPv6」「對外連線 DNS 抖動」兩節為準**；這裡保留證據、
被推翻的歸因與推導，免得下一個人重走一遍。內文的「上一節 / 下一節」指的是本檔內的相對位置。

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
`get_json_data`（＝ TWSE 全部路徑）與 `services/lotto.rs`（該檔已隨樂透功能移除）、`services/gov_tenders.rs`，
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

### 三處改動

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
    for h in www.twse.com.tw pcc-api.openfun.app oauth2.googleapis.com; do
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
