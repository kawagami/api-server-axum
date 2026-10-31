# 檔案直傳 — WS Signaling 協定（前後端契約）

> **本檔是前後端唯一的協定來源，進版控。改協定＝同一個 commit 改兩邊 + 改本檔。**
>
> 後端實作：`backend/src/services/ws/transfer.rs`（分派入口 `services/ws/socket.rs::process_message`）。
> 前端實作：`frontend/app/admin/(main)/transfer/`（`useFileTransfer.ts` 狀態機、`transfer-model.ts` 型別與 DataChannel 格式）。

後台「檔案直傳」頁（`/admin/transfer`）讓同一個 admin 帳號的兩台裝置（手機 ↔ PC）互傳檔案。
**檔案走瀏覽器之間的 WebRTC DataChannel，不經後端**；後端只負責轉發建立連線用的 SDP。

## 設計取捨

- **只用 STUN（Google / Cloudflare 公開伺服器），不設 TURN**。同一個區網一定通、多數家用網路靠 STUN 打洞通；
  行動網路的 CGNAT / symmetric NAT 常打不通 —— 打不通就在前端告知失敗，**檔案永遠不經伺服器**。
  要補 TURN 只需在前端 `ICE_SERVERS` 加一筆，協定不變。
- **不做 trickle ICE**：兩端都等 ICE 候選收集完（上限 3 秒）才把整份 SDP 送出，所以 signaling 只有 offer / answer 兩則，
  不會碰到每連線收訊令牌桶（容量 30、每秒回補 6）。
- **配對靠帳號**：只在 `user_name` 相同的 admin 連線之間轉發。admin 身分來自 WS ticket（要求 `ws:read`），
  所以頁面權限也是 `ws:read`，後端不另查權限。匿名連線不能進房。
- **一次一個傳輸**：處理中收到新的 offer，前端自動回 `transfer_reject { reason: "busy" }`。
- 功能開關 key：`file_transfer`（`Feature::FileTransfer`）。關閉時所有 `transfer_*` 上行回 `transfer_error { reason: "feature_disabled" }`。

## 信封

一般 WS 信封 `{ type, data }`，**不帶 `game` 欄**（帶 `game` 的會被分派到遊戲 hub）。
這些 type 不經 `WsEvent` enum、不進前端 `WsEventType`，不會觸發全站 toast（同遊戲訊息）。

## 上行（Client → Server）

| type | data | 說明 |
|------|------|------|
| `transfer_join` | 無 | 進傳輸頁。WS 重連後要重送（server 端房間狀態隨連線消失） |
| `transfer_leave` | 無 | 離開傳輸頁 |
| `transfer_offer` | `{ to, id, sdp, files: [{ name, size, type }] }` | 送出端發起；`id` 由送出端產生（UUID），整個傳輸共用 |
| `transfer_answer` | `{ to, id, sdp }` | 接收端接受 |
| `transfer_reject` | `{ to, id, reason? }` | 拒絕 / 取消（任一方、連上之前）。`reason: "busy"` = 對方正在處理另一個傳輸 |

`to` = 對方連線的 `addr`（取自 `transfer_peers`）。三種轉發訊息後端都**原樣轉發**，只把 `to` 換成 `from`；
`data` 序列化後超過 64KB 回 `too_large`。後端不追蹤傳輸狀態（offer → answer 的順序由前端保證）。

## 下行（Server → Client）

| type | data | 說明 |
|------|------|------|
| `transfer_peers` | `{ peers: [{ addr, user_agent, connected_at }] }` | 同帳號、在房的其他連線（不含自己）。有人加入 / 離開 / 斷線時推給同帳號的所有在房連線 |
| `transfer_offer` | `{ from, id, sdp, files }` | 轉發 |
| `transfer_answer` | `{ from, id, sdp }` | 轉發 |
| `transfer_reject` | `{ from, id, reason? }` | 轉發 |
| `transfer_error` | `{ reason }` | 只回給送出那則訊息的連線 |

`transfer_error.reason`：

| reason | 時機 |
|--------|------|
| `feature_disabled` | instance 關閉 `file_transfer` |
| `not_admin` | 匿名連線送 `transfer_join` / `transfer_leave` |
| `not_joined` | 送出端不是在房的 admin 連線 |
| `peer_not_found` | `to` 不存在、格式錯、不在房、不同帳號，或是自己 |
| `too_large` | 轉發的 `data` 超過 64KB |

## 流程

```
A(送) ── transfer_offer {to:B, id, sdp, files} ──▶ server ──▶ B(收)    B 顯示「接受 N 個檔案？」
A     ◀── transfer_answer {from:B, id, sdp} ◀── server ◀── B           B 按接受
            ═════ WebRTC DataChannel 直連（15 秒內沒連上 → 兩端各自顯示「無法直連」）═════
A ── {kind:"file",index:0} → ArrayBuffer… → {kind:"file",index:1} → … → {kind:"done"} ──▶ B
A ◀────────────────────────────────── {kind:"ack"} ────────────────────────────────── B
A 收到 ack 後關閉連線；B 的 channel close 時收尾
```

## DataChannel 格式（只有前端兩端在用）

- 單一 channel `files`，ordered + reliable，`binaryType = "arraybuffer"`
- 文字訊息是控制：`{ kind: "file", index }`（接下來的 binary 屬於 offer `files[index]`）、`{ kind: "done" }`、`{ kind: "ack" }`
- binary 是檔案內容，每則 ≤ 64KB（與 `pc.sctp.maxMessageSize` 取小）。接收端累計到 `files[index].size` 就組成一個 `File`
- 背壓：`bufferedAmount` 超過 8MB 停送，等 `bufferedamountlow`（門檻 1MB）
- 已知限制：接收端整份檔案暫存在記憶體（總量超過 1GB 時接受前會警告），沒有斷點續傳
