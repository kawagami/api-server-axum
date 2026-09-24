# 對戰遊戲 — WS Wire 協定（前後端契約）

> **本檔是前後端唯一的協定來源，進版控。** 2026-09-25 由原本只存在本機、不進版控的 `docs/old/` 四份文件整合而來
> （`chess-wire-protocol.md` + 西洋棋/圍棋、阿瓦隆、農場三份前端交付說明）。各章節內容照搬原文，
> 只改了三處已確認過時的敘述（遊戲清單、legacy echo、「本檔讀不到」），並拿掉已全數完成的「前端 TODO」清單。
>
> 後端實作：`backend/src/games/common/`（共用框架）+ `backend/src/games/<game>/`；分派入口 `backend/src/services/ws/socket.rs::dispatch_game`。
> 前端實作：`frontend/app/[locale]/(public)/games/_shared/`（2 人框架）與各遊戲目錄。
> **改協定＝同一個 commit 改兩邊 + 改本檔。**

## 目前狀態（2026-09-25 對照程式碼核對）

| 遊戲 | `game` | 框架 | 章節 |
|------|--------|------|------|
| 象棋 | `chess` | 2 人（大廳 / 桌位 / 配對 / Fischer） | 一 |
| 五子棋 | `gomoku` | 同上 | 一（附錄） |
| 暗棋 | `banqi` | 同上 | 一（附錄） |
| 西洋棋 | `western_chess` | 同上 | 二 |
| 圍棋 | `go` | 同上 | 二 |
| 阿瓦隆 | `avalon` | N 人房（`common/room.rs`） | 三 |
| 農場經營 | `farm` | N 人房 | 四 |

原文寫成之後（2026-06-19）才加上、原文沒有的協定內容：

- **`hints`（下行，2 人框架，2026-08-24）**：server 在 `match_found` 之後與每次 `move_made` 之後，**只推給當前輪到的那一方**（同批送出，順序有保證；對局已結束不推）。**不是權威**，判定仍只有 server 的走步驗證。形狀各遊戲自定：象棋 / 西洋棋 / 暗棋 = `{ moves: { "col,row": [[col,row]…] } }`（暗棋另有 `flips`）、圍棋 = `{ forbidden: [[col,row]…] }`（只送禁著點）、五子棋不送。
- **`error.reason` 新增**：`too_many_tables`（2 人框架，桌數達上限 200，`create_table` / `join_queue` 都會回）、`too_many_rooms`（N 人房，房數達上限 100）、`feature_disabled`（instance 關閉 `games` 功能時，所有遊戲訊息都回這個，帶 `game`）。
- **連線層限流**：單條連線收訊令牌桶（容量 30、每秒回補 6），超量的訊息直接丟棄，第一則丟棄時回一則 `{type:"error",data:{reason:"rate_limited"}}`（**不帶 `game`**），累計丟 200 則才收線。per-IP 匿名連線上限 12 條，超過在握手時回 HTTP 429。
- **心跳**：server 每 30 秒送 WS Ping，75 秒沒收到 Pong 就判定連線已死並走斷線流程（瀏覽器會自動回 Pong，前端不必處理）。

---

## 一、2 人框架通用協定 + 象棋 / 五子棋 / 暗棋

> Server-authoritative。所有規則 / 計時 / 勝負判定在 server，前端只負責顯示與送指令。
> 本檔為前後端唯一契約來源；後端實作見 `src/games/common/`（共用框架）+ `src/games/<game>/`（各遊戲引擎 + adapter）。
> 涵蓋遊戲：**象棋 `chess`**、**五子棋 `gomoku`**、**暗棋 `banqi`**。大廳 / 桌位 / 配對 / 計時 / 斷線**三遊戲完全共用**，只差「座標系 / move 內容 / 顏色標籤 / reason」。

### ⚠️ 前端注意（migration / 上手指引）

1. **象棋是破壞性改動，不是純新增**：信封新增**必填** `game` 欄。已上線的象棋程式碼要改 —— 送訊每則加 `"game":"chess"`，收訊先以 `game` 欄過濾。沒帶 `game` 的訊息 server 不再當遊戲處理。
2. **本檔只含 WS 協定，不含棋規**：象棋棋規以後端引擎 `backend/src/games/chess/engine.rs`（附測試）為準（原設計稿 `chess-multiplayer-spec.md` 未進版控，已被引擎取代）；五子棋＝五連（含以上）勝，本檔已足；暗棋位階 / 吃子 / 翻子 / 炮規則見文末「各遊戲差異附錄」。
3. **棋盤 UI 前端自繪**（尺寸見各遊戲段）：象棋 9×10、五子棋 15×15、暗棋 8 欄×4 列（32 子面朝下）。
4. **暗棋顏色延後揭示**：`match_found.color` 是座位 `first`/`second`，不是紅黑；紅黑要等首次翻子的 `move_made.data.piece.color` 才確定。
5. ~~本檔在後端 repo 且 `.gitignore`，前端 repo 讀不到~~ → 2026-09-25 起進版控（monorepo 根 `protocol/games-wire.md`），前後端共讀這一份。

### 連線與信封

- 端點：`ws://<host>/ws`（與站上既有 WS 共用，**匿名可玩**，不需 JWT）。
- 信封一律 `{ "game": <string>, "type": <string>, "data": <object|null> }`。
- **`game` 欄必填**，值 ∈ `"chess"` / `"gomoku"` / `"banqi"` / `"western_chess"` / `"go"` / `"avalon"` / `"farm"`（2026-09-25 對照 `backend/src/games/registry.rs`）。上行要帶、下行也會帶回。一條連線同一時間只玩一個遊戲，但 socket 共用，故靠 `game` 分流。
- ⚠️ **同一條 WS 會收到無關訊息**：全站通知事件（`stock_completed`、`admin_message`…；`user_joined` / `user_left` 現在只推給 admin 連線）、連線層的 `{type:"error",data:{reason:"rate_limited"}}`（**不帶 `game`**）。~~非 JSON 的 legacy echo~~ 已移除：server 收到非 JSON / 未知訊息一律忽略、不回應。**前端用 `game` + `type` 白名單過濾，不符的一律丟棄。**
- 下方範例為精簡多半只標象棋；**實際每則都必須帶對應 `game` 欄**（如 `"game":"chess"`）。

### 座標系

- `[col, row]` 數字陣列。`col` 0–8（左→右），`row` 0–9（下→上）。**絕對座標，全程不翻轉。**
- 紅方在下（row 0–4），黑方在上（row 5–9）。河界在 row 4 / 5 之間。
- 前端「黑方視角翻轉」純屬渲染；送 server 的 `from` / `to` 永遠是絕對座標。
- color / turn / side 字串一律小寫 `"red"` / `"black"`。

### 初始局面（前端自行擺，server 不送盤面）

紅方（下）：俥`[0,0][8,0]` 傌`[1,0][7,0]` 相`[2,0][6,0]` 仕`[3,0][5,0]` 帥`[4,0]` 炮`[1,2][7,2]` 兵`[0,3][2,3][4,3][6,3][8,3]`
黑方（上）：車`[0,9][8,9]` 馬`[1,9][7,9]` 象`[2,9][6,9]` 士`[3,9][5,9]` 將`[4,9]` 砲`[1,7][7,7]` 卒`[0,6][2,6][4,6][6,6][8,6]`

`move_made` 只送 `from`/`to`，前端自行套用走步維護盤面。

---

### 上行（Client → Server）

| type | data | 說明 |
|------|------|------|
| `join_lobby` | – | 進象棋頁送一次。訂閱大廳更新，server 立即回 `table_list` |
| `list_tables` | – | 主動索取當前桌列表（回 `table_list`） |
| `create_table` | `{ "name"?: string }` | 自建空桌等對手。name 選填、自動 trim、≤40 字、空則命名 `桌 #<id>` |
| `join_table` | `{ "table_id": number }` | 坐進等待中的桌，湊滿即開局 |
| `leave_table` | – | 等待中的 host 離開、銷毀自己的桌（**對戰中改用 `resign`**） |
| `join_queue` | – | 快速隨機配對，入單一佇列；湊滿 2 自動開局 |
| `leave_queue` | – | 離開配對佇列 |
| `move` | `{ "from": [c,r], "to": [c,r] }` | 行棋 |
| `resign` | – | 認輸 |

**承諾互斥**：一條連線同時只能在「佇列」或「某張桌」其一。已有承諾時再送 `create_table`/`join_table`/`join_queue` → `create/join` 回 `error{already_committed}`，`join_queue` 直接忽略。

### 下行（Server → Client）

| type | data | 對象 | 說明 |
|------|------|------|------|
| `table_list` | `{ tables: Table[] }` | 請求者 | 回應 `join_lobby` / `list_tables` |
| `lobby_update` | `{ tables: Table[] }` | 所有大廳訂閱者 | 桌況變動（建桌/開局/結束/銷毀）時推播 |
| `table_created` | `{ table_id }` | host | 建桌成功 |
| `queued` | `{ position }` | 請求者 | 已入快速佇列（position = 佇列長度） |
| `match_found` | `{ color, clock_ms, table_id }` | 對局雙方 | 配對成功，分紅/黑 + 初始時鐘。**收到即進棋盤畫面** |
| `move_made` | `{ from, to, turn, clock }` | 對局雙方 | 合法走步後廣播 |
| `check` | `{ side }` | 對局雙方 | 被將軍提示（緊接在 `move_made` 之後） |
| `illegal_move` | `{ reason }` | 行棋方 | 非法走步，盤面不動 |
| `game_over` | `{ winner, reason }` | 對局雙方 | 對局結束，桌已銷毀 |
| `error` | `{ reason }` | 觸發者 | 防呆兜底 |

#### Table 物件
```json
{ "id": 3, "name": "桌 #3", "status": "waiting" }
```
- `status`：`"waiting"`（可 join）/ `"playing"`（對戰中）。列表依 `id` 升序。

---

### JSON 範例

進大廳：
```json
// → join_lobby
{ "type": "join_lobby" }
// ← table_list
{ "type": "table_list", "data": { "tables": [
  { "id": 1, "name": "高手局", "status": "playing" },
  { "id": 2, "name": "桌 #2", "status": "waiting" }
] } }
```

自建桌：
```json
// → create_table
{ "type": "create_table", "data": { "name": "來玩" } }
// ← table_created（給 host）
{ "type": "table_created", "data": { "table_id": 5 } }
// ← lobby_update（給所有大廳訂閱者）
{ "type": "lobby_update", "data": { "tables": [ /* …含新桌 status:waiting */ ] } }
```

挑桌入座 → 開局：
```json
// → join_table
{ "type": "join_table", "data": { "table_id": 5 } }
// ← match_found（分別送雙方，color 不同）
{ "type": "match_found", "data": { "color": "red", "clock_ms": 300000, "table_id": 5 } }
{ "type": "match_found", "data": { "color": "black", "clock_ms": 300000, "table_id": 5 } }
```

快速配對：
```json
// → join_queue
{ "type": "join_queue" }
// ← queued（還沒湊到對手時）
{ "type": "queued", "data": { "position": 1 } }
// ← match_found（湊到對手，同上格式，table_id 為自動建立的快速對局桌）
```

行棋：
```json
// → move
{ "type": "move", "data": { "from": [1,2], "to": [4,2] } }
// ← move_made（雙方）
{ "type": "move_made", "data": {
  "from": [1,2], "to": [4,2],
  "turn": "black",
  "clock": { "red": 312000, "black": 300000 }
} }
// ← check（若這步造成將軍，緊接著送；side = 被將方 = 該走方）
{ "type": "check", "data": { "side": "black" } }
```

結束：
```json
{ "type": "game_over", "data": { "winner": "red", "reason": "checkmate" } }
// 和棋 winner 為 null
{ "type": "game_over", "data": { "winner": null, "reason": "draw_60" } }
```

---

### 計時（標準 Fischer，server 權威）

- 初始各 `300000 ms`（5:00），每走完一步該方 `+30000 ms`。
- `match_found` 推出那刻起紅方倒數（紅先行）。
- server 收到 `move` 時：`elapsed = now − 本回合開始`，扣行棋方時鐘 → ≤0 判 `timeout` 負；否則 `+30s`、切換、重置起點。
- **網路延遲算行棋方頭上**（以 server 收到時刻為準）。
- `move_made.clock` 帶雙方剩餘 ms，前端本地倒數顯示即可，判定全在 server。
- 行棋方坐著不走也會超時：server 每秒掃描，耗盡即主動推 `game_over{timeout}`。

### reason 列舉

`game_over.reason`（小寫 snake / 單字）：
`checkmate` / `stalemate`（困斃，被困方判負）/ `timeout` / `resign` / `disconnect` / `draw_60`

`illegal_move.reason`（⚠️ 大小寫混合）：
- 引擎規則（PascalCase）：`NotYourTurn` `NoPiece` `WrongPiece` `CaptureOwn` `BadMove` `BlockedPath` `BadHorseLeg` `BadElephantEye` `CrossRiver` `OutOfPalace` `FlyingGeneral` `LeavesKingInCheck`
- 協定層（snake）：`bad_coord`

`error.reason`（snake）：
`already_committed` / `bad_table_id` / `table_not_found` / `table_full` / `cannot_join_self` / `not_in_game` / `game_ended`

### 斷線

WS close → server 偵測：
- 在佇列 → 移除。
- 在等待中的桌（host）→ 銷毀桌、推 `lobby_update`。
- 對戰中 → 對手以 `game_over{ reason: "disconnect" }` 判勝，桌銷毀。

無重連窗口（斷線即判敗，重啟 server＝所有對局丟失）。

---

### 附錄：五子棋 / 暗棋與象棋的差異

上方「座標系 / 初始局面 / check / reason 列舉」皆**象棋**。大廳 / 桌位 / 配對 / `match_found` / `move_made` 外殼（`turn` + `clock`）/ `game_over` / `error` / 斷線/計時**三遊戲共用**。共用的 harness 級 `game_over.reason`：`timeout` / `resign` / `disconnect`；`error.reason`：`already_committed` / `bad_table_id` / `table_not_found` / `table_full` / `cannot_join_self` / `not_in_game` / `game_ended` / `bad_coord`。

`move_made.data` 一律含 `turn`（下一手輪到誰，值見各遊戲 color 標籤）與 `clock`（鍵 = 兩方 color 標籤，值剩餘 ms），外加各遊戲自有欄位（見下）。`match_found.data.color` = 我方 color 標籤。

#### 五子棋 `gomoku`

- 棋盤 15×15，座標 `[col,row]`，皆 0–14。黑先。
- color 標籤：`"black"`（先手）/ `"white"`。`clock` 鍵為 `black`/`white`。
- 上行 `move`：`{ "at": [col,row] }`（落子）。
- `move_made.data`：`{ "at":[c,r], "by": "black"|"white", "turn":..., "clock":{...} }`（`by` = 剛落子方）。
- `game_over.reason`：`five_in_row`（五連含以上）/ `draw_full`（滿盤）/ + 共用 timeout/resign/disconnect。
- `illegal_move.reason`：`bad_coord` / `occupied`。
- 無 `check` 事件。

範例：
```json
// → { "game":"gomoku", "type":"move", "data":{ "at":[7,7] } }
// ← { "game":"gomoku", "type":"move_made", "data":{
//      "at":[7,7], "by":"black", "turn":"white",
//      "clock":{ "black":312000, "white":300000 } } }
// ← { "game":"gomoku", "type":"game_over", "data":{ "winner":"black", "reason":"five_in_row" } }
```

#### 暗棋（翻棋）`banqi`

- 棋盤 8 欄 × 4 列，座標 `[col,row]`，col 0–7、row 0–3。象棋全套 32 子面朝下隨機洗牌（server 持有，翻開才揭示）。
- color 標籤用**座位** `"first"`（先手）/ `"second"`；紅黑由**首次翻子**決定（翻子者執翻出色），於事件 `piece.color` 揭示。`clock` 鍵為 `first`/`second`。
- 第一手必須翻子（尚未定色時送 move 回 `illegal_move{no_color_yet}`）。
- 上行 `move` 兩種 `action`：
  - 翻子：`{ "action":"flip", "at":[col,row] }`
  - 走/吃：`{ "action":"move", "from":[c,r], "to":[c,r] }`
- `move_made.data`：
  - 翻子：`{ "action":"flip", "at":[c,r], "piece":{ "color":"red"|"black", "kind":<kind> }, "turn":..., "clock":{...} }`
  - 走/吃：`{ "action":"move", "from":[c,r], "to":[c,r], "captured": null | { "color","kind" }, "turn":..., "clock":{...} }`
- `kind` ∈ `king`/`guard`/`elephant`/`rook`/`horse`/`cannon`/`pawn`（位階高→低）。吃子：相鄰且階 ≤ 己；例外 `pawn` 可吃 `king`、`king` 不可吃 `pawn`；`cannon` 隔一架同線吃任意階（僅吃已翻開敵子）。
- `game_over.reason`：`elimination`（對方子全亡）/ `no_moves`（輪到方無步）/ `draw_quiet`（連續 60 半步無吃子）/ + 共用 timeout/resign/disconnect。
- `illegal_move.reason`：`bad_coord` / `bad_action` / `no_color_yet` / `not_hidden` / `already_up` / `not_revealed` / `empty_from` / `not_your_piece` / `not_adjacent` / `occupied_friendly` / `cannot_capture` / `target_hidden` / `cannon_needs_screen` / `bad_cannon_move` / `blocked`。
- 無 `check` 事件。

範例：
```json
// 翻子
// → { "game":"banqi", "type":"move", "data":{ "action":"flip", "at":[0,0] } }
// ← { "game":"banqi", "type":"move_made", "data":{
//      "action":"flip", "at":[0,0], "piece":{ "color":"red","kind":"cannon" },
//      "turn":"second", "clock":{ "first":312000, "second":300000 } } }
// 吃子
// → { "game":"banqi", "type":"move", "data":{ "action":"move", "from":[0,0], "to":[1,0] } }
// ← { "game":"banqi", "type":"move_made", "data":{
//      "action":"move", "from":[0,0], "to":[1,0],
//      "captured":{ "color":"black","kind":"pawn" }, "turn":"first", "clock":{...} } }
```

---

## 二、西洋棋 + 圍棋

> 日期：2026-06-19
> 對象：前端（`blog-next`）
> 摘要：後端新增兩款遊戲，都接在既有共用對戰框架上。**大廳 / 桌位 / 配對 / 計時 / 斷線 / game_over 流程與象棋完全相同**，差別只在 `game` id、座標系、move 內容、顏色標籤、reason。本檔只列差異；通用流程見 `chess-wire-protocol.md`。

---

### 共通（與現有三遊戲一致，不重述）

- 信封 `{ game, type, data }`。
- 上行：`join_lobby` / `list_tables` / `create_table` / `join_table` / `leave_table` / `join_queue` / `leave_queue` / `move` / `resign`。
- 下行：`table_list` / `lobby_update` / `table_created` / `queued` / `match_found` / `move_made` / `game_over` / `error`。
- `match_found.data` = `{ color, clock_ms, table_id }`；`color` = 該玩家的座位標籤（見各遊戲）。
- `move_made.data` 一定含 `turn`（走完後輪到誰，座位標籤）＋ `clock`（鍵為兩座位標籤，值剩餘 ms）。其餘欄位各遊戲不同（見下）。
- 計時：Fischer 初始 5:00、每步 +30s。
- ⚠️ 切離遊戲頁時前端要主動送 `leave_table` / `leave_queue` / `resign`（共用 socket 不會因切頁斷線）。

---

### 一、西洋棋 `game = "western_chess"`

- 棋盤 8×8。座標 `[col, row]`，col 0–7（a–h），row 0–7，**白方底線 row 0**。絕對座標，前端翻轉純渲染。
- 座位標籤：`white`（先手，First）/ `black`。
- 初始局面：標準擺法（白 row 0–1、黑 row 6–7），前端自繪。

#### 上行 move
```jsonc
{ "game": "western_chess", "type": "move",
  "data": { "from": [4,1], "to": [4,3] } }
// 升變（兵到底線才需）：promo 為 "q"|"r"|"b"|"n"，省略預設 "q"
{ "game": "western_chess", "type": "move",
  "data": { "from": [0,6], "to": [0,7], "promo": "q" } }
```

#### 下行 move_made.data
```jsonc
{
  "from": [4,1], "to": [4,3],
  "turn": "black",
  "clock": { "white": 312000, "black": 300000 },
  // 以下為選填特殊旗標，前端據此額外處理：
  "promo": "q",          // 此步為升變 → 把 to 的兵換成該子
  "castle": "king",      // 王車易位（"king"/"queen"）→ 對應車也要移動（王翼 h↔f、后翼 a↔d）
  "ep_capture": [3,4]    // 吃過路兵 → 移除此格的兵（不是 to 格）
}
```
> 前端套用走步：移動 from→to；若有 `castle` 連帶移動車；若有 `ep_capture` 移除該格兵；若有 `promo` 替換棋子。

#### 下行 check
```json
{ "game": "western_chess", "type": "check", "data": { "side": "white" } }
```
緊接在造成將軍的 `move_made` 之後（`side` = 被將方）。

#### game_over.reason
`checkmate`（`winner` = 勝方）／ `stalemate`（逼和，`winner: null`）／ `draw`（50 步或子力不足，`winner: null`）／ `timeout` / `resign` / `disconnect`。

#### illegal_move.reason
`bad_coord` / `no_piece`（起點無子）/ `wrong_piece`（起點非己方子）/ `illegal`（不合規）/ `bad_promo`。

---

### 二、圍棋 `game = "go"`

- 棋盤 19×19，棋子下在**交叉點**。座標 `[col, row]`，0–18。
- 座位標籤：`black`（先手，First）/ `white`。
- 貼目 komi 7.5（內建，前端不用算）。

#### 上行 move
```jsonc
// 落子
{ "game": "go", "type": "move", "data": { "at": [3,3] } }
// 虛手（pass）
{ "game": "go", "type": "move", "data": { "pass": true } }
```
> 前端需提供「虛手 pass」按鈕。**連兩次虛手 → 終局數子**。

#### 下行 move_made.data
```jsonc
// 落子
{
  "at": [3,3], "by": "black",
  "captured": [[3,4],[4,4]],   // 本手提走的對方棋子座標 → 前端從盤面移除
  "turn": "white",
  "clock": { "black": 312000, "white": 300000 }
}
// 虛手
{ "pass": true, "by": "black", "turn": "white", "clock": { "black": 300000, "white": 300000 } }
```

#### game_over.reason
`score`（連兩虛手終局，`winner` = 數子＋貼目較高方）／ `timeout` / `resign` / `disconnect`。
> 後端只回 `winner`；目前**不回雙方目數明細**（要顯示比分再跟後端加）。

#### illegal_move.reason
`bad_coord` / `out_of_bounds` / `occupied` / `ko`（劫爭禁著）/ `suicide`（自殺手）。

---

## 三、阿瓦隆 Avalon

> 日期：2026-06-19
> 對象：前端（`blog-next`）
> 摘要：社交推理 N 人遊戲。**與既有 2 人對戰框架完全不同**：N 人房、私有角色、階段機、投票、內建聊天。共用同一條 `/ws` 與信封 `{ game, type, data }`（`game = "avalon"`），但**不用** chess/go 那套大廳/桌位/Fischer 計時流程。本檔是 avalon 完整協定。

### ⚠️ 開工前先記三件事

1. **配 `chess-wire-protocol.md` 兩條全站通則**：① 同條 `/ws` 會收到非 avalon 訊息（通知、其他遊戲），前端要先以 `game === "avalon"` 過濾；② **切離遊戲頁要主動送 `leave_room`**（共用 socket 不會因切頁自動斷線，否則房卡人）。
2. **私有角色絕不可外洩**：每人的 `role_assigned` 只屬於自己（`known` 各不相同）。**千萬別把角色放進共用 state 再渲染給全房** —— 這是阿瓦隆最致命的雷。
3. **要懂阿瓦隆規則才寫得出 UI 文案**：本檔給協定 + `known` 語意 + 階段，但「梅林看到什麼提示、刺客階段給誰按鈕」需理解遊戲，不熟請先讀阿瓦隆規則。

---

### 連線與信封

- 端點：`ws://<host>/ws`（與全站共用，匿名）。
- 信封：`{ "game": "avalon", "type": ..., "data": ... }`。收訊先濾 `game === "avalon"`。
- 無計時（討論型，不限時）。

### 核心概念

- **5–10 人**。房主(host)滿 5 人即可開局。
- **私有角色**：開局時每位玩家收到**只屬於自己**的 `role_assigned`（看到的 `known` 各不相同）。⚠️ 絕不可把某人的角色廣播給全房。
- **座位 seat**：玩家以 `seat`（0..n-1，加入順序）識別，全程用 seat 指涉（提名、投票、刺殺）。
- **無淘汰**：所有人玩到結束。

### 階段機

```
team_building : 隊長提名 quest_size 人
   ↓ (propose_team)
team_vote     : 全員公開投票贊成/否決
   ↓ 過半贊成 → quest ；否決 → 換隊長回 team_building（連 5 次否決 → 壞人勝）
quest         : 上場者暗投 成功/失敗（好人只能投成功）
   ↓ 成功任務達 3 → assassinate ；失敗任務達 3 → 壞人勝
assassinate   : 刺客猜梅林 → 猜中壞人逆轉勝，否則好人勝
   ↓
game_over     : 揭露全角色
```

---

### 上行（Client → Server）

#### 大廳 / 房間（開局前）
| type | data | 說明 |
|------|------|------|
| `join_lobby` | – | 訂閱大廳，回 `room_list` |
| `list_rooms` | – | 取房列表 |
| `create_room` | `{ room_name?, nickname?, options?: { mordred?, oberon? } }` | 建房，自己當 host（seat 0）。`room_name`=房名（≤40，預設「房 #id」）；`nickname`=自己暱稱（≤20） |
| `join_room` | `{ room_id, nickname? }` | 加入房。`nickname`=自己暱稱（≤20） |
| `leave_room` | – | 離開房（host 離開 → 解散房） |
| `start_game` | – | **僅 host**，5–10 人時開局 |
| `chat` | `{ text }` | 房內聊天（≤500 字） |

> 欄位分明：**`room_name`=房名、`nickname`=玩家暱稱**（兩者不再共用 `name`）。暱稱留空 → 後端預設「玩家N」（N=座位+1，host=玩家1）。host 與一般玩家都可設暱稱。

#### 對局中
| type | data | 說明 |
|------|------|------|
| `propose_team` | `{ team: [seat,...] }` | **僅當前隊長**，人數須等於 `quest_size` |
| `team_vote` | `{ approve: bool }` | 每人一票 |
| `quest_card` | `{ success: bool }` | **僅上場者**；好人投 `false` 會被拒 |
| `assassinate` | `{ target: seat }` | **僅刺客**，於 assassinate 階段 |

### 下行（Server → Client）

| type | data | 對象 | 說明 |
|------|------|------|------|
| `room_list` | `{ rooms:[{id,name,players,max,status}] }` | 請求者 | |
| `lobby_update` | `{ rooms:[...] }` | 大廳訂閱者 | 房況變動 |
| `room_created` | `{ room_id }` | host | |
| `room_update` | `{ room_id, name, host_seat, players:[{seat,name}], options, can_start }` | 房內成員 | 有人進出/可否開局 |
| `room_closed` | `{ reason }` | 房內其餘成員 | host 離開 / 對局中止（`host_left`/`aborted`） |
| **`role_assigned`** | `{ your_seat, your_role, known:[seat,...], n, sizes:[5], players:[{seat,name}] }` | **逐人私有** | 開局時，各人不同 |
| `phase_changed` | `{ phase, leader, round, quest_size, results:[bool], rejects, team:[seat,...] }` | 全房 | 階段轉換 |
| `team_proposed` | `{ team:[seat,...], leader }` | 全房 | 隊長提名了 |
| `vote_result` | `{ votes:[{seat,approve}], approved }` | 全房 | 組隊投票**公開**結果 |
| `quest_result` | `{ round, fails, success }` | 全房 | 任務結果（`fails`=失敗票數，**不公開誰投的**） |
| `game_over` | `{ winner: "good"\|"evil"\|null, reason, roles:[{seat,role}] }` | 全房 | 揭露全角色 |
| `chat` | `{ seat, name, text }` | 全房 | |
| `error` | `{ reason }` | 觸發者 | |

#### role_assigned 的 `known` 意義（依 `your_role`）
- `merlin` → `known` = 所有壞人座位（**除莫德雷 mordred**）
- `percival` → `known` = 梅林與莫甘娜的座位（**無法分辨誰是誰**）
- `assassin`/`morgana`/`mordred`/`minion`（非奧伯倫）→ `known` = 其他壞人座位（**除奧伯倫**）
- `loyal_servant`/`oberon` → `known` = 空

角色字串：`merlin` `percival` `loyal_servant` `assassin` `morgana` `mordred` `oberon` `minion`。

---

### JSON 範例

開局（host 送 `start_game` 後，server 對**每人**送不同 role_assigned，再對全房送 phase_changed）：
```jsonc
// → 梅林那位收到
{ "game":"avalon", "type":"role_assigned",
  "data":{ "your_seat":0, "your_role":"merlin", "known":[3,4], "n":5, "sizes":[2,3,2,3,3],
           "players":[{"seat":0,"name":""},{"seat":1,"name":"阿明"}, /*…*/ ] } }
// ← 全房
{ "game":"avalon", "type":"phase_changed",
  "data":{ "phase":"team_building", "leader":2, "round":0, "quest_size":2,
           "results":[], "rejects":0, "team":[] } }
```

組隊 → 投票 → 任務：
```jsonc
// 隊長(seat2) → propose_team
{ "game":"avalon", "type":"propose_team", "data":{ "team":[2,4] } }
// ← team_proposed（全房）+ phase_changed(team_vote)
{ "game":"avalon", "type":"team_proposed", "data":{ "team":[2,4], "leader":2 } }
// 每人 → team_vote
{ "game":"avalon", "type":"team_vote", "data":{ "approve":true } }
// ← vote_result（全部投完才送）
{ "game":"avalon", "type":"vote_result",
  "data":{ "votes":[{"seat":0,"approve":true},/*…*/], "approved":true } }
// 過半 → phase_changed(quest)；上場者(2,4) → quest_card
{ "game":"avalon", "type":"quest_card", "data":{ "success":false } }
// ← quest_result（全上場者投完）
{ "game":"avalon", "type":"quest_result", "data":{ "round":0, "fails":1, "success":false } }
```

結局：
```jsonc
{ "game":"avalon", "type":"game_over",
  "data":{ "winner":"evil", "reason":"evil_assassinate",
           "roles":[{"seat":0,"role":"merlin"},{"seat":3,"role":"assassin"}, /*…*/ ] } }
```

---

### 人數配置（前端顯示用，後端已內建）
| 人數 | 好/壞 | 任務人數(5輪) |
|------|-------|---------------|
| 5 | 3/2 | 2 3 2 3 3 |
| 6 | 4/2 | 2 3 4 3 4 |
| 7 | 4/3 | 2 3 3 4 4 |
| 8–10 | 5–6/3–4 | 3 4 4 5 5 |
> 7 人以上的**第 4 輪**（round index 3）需 **2 張失敗票**才算任務失敗（其餘 1 張即敗）。

### reason 列舉
- `game_over.reason`：`good_three_quests`(無刺客時，本設定恆有刺客故少見) / `good_assassin_miss` / `evil_three_fails` / `evil_five_rejects` / `evil_assassinate`
- `room_closed.reason`：`host_left` / `aborted`（對局中有人斷線→中止）
- `error.reason`：`already_committed` / `bad_room_id` / `room_not_found` / `room_full` / `already_started` / `not_in_room` / `not_host` / `cannot_start` / `not_in_game` / `not_leader` / `bad_team` / `bad_team_size` / `wrong_phase` / `bad_vote` / `bad_card` / `not_on_team` / `good_must_succeed` / `bad_target` / `not_assassin` / `too_many_special_evil` / `bad_player_count`

### 斷線
- 大廳：移除訂閱。
- 房內等待中：移除座位（host 斷線 → 解散房，其餘收 `room_closed{host_left}`）。
- **對局中任一人斷線 → 整局中止**（其餘收 `room_closed{aborted}`，房解散）。無重連。

---

## 四、農場經營 Farm（家庭版 worker-placement）

> 日期：2026-06-19
> 對象：前端（`blog-next`）
> 摘要：2–4 人、**完全資訊**的 worker-placement 經營遊戲（原創機制，非任何商業桌遊的素材/文案）。沿用 `/ws` 與信封 `{ game:"farm", type, data }`。大廳/房流程與阿瓦隆相同；**對局每次動作後 server 廣播完整盤面**（無私有狀態）。本檔是 farm 完整協定。

### 開工前（必讀）

- **⚠️ 無精確格子座標 → 只能抽象呈現**：phase-1 引擎把牧場抽象成「**N 格一塊**」，房/田/牧場只記**數量與內容**，不記「哪一格是什麼」。`state` 給的是 `rooms:2`、`fields:[{crop,count}|null]`、`pastures:[{tiles,stable,animal}]`、`loose_stables`、`free_tiles`。前端**畫不出逐格農莊**，請用圖示 + 數量抽象呈現（例：「2 房 / 田 1（穀×3）/ 牧場 1（2 格・羊×3）/ 空格 11」）。要逐格盤面＝第二期後端補座標。**這是 farm 與棋類最大差異，先設計好抽象 UI。**
- 配 `chess-wire-protocol.md` 兩通則：① 收訊先以 `game === "farm"` 過濾；② **切離遊戲頁主動送 `leave_room`**（共用 socket 不自動斷線）。
- **複合動作要 input 表單**：`sow` / `build_rooms` / `fences` 需參數輸入 UI（見下方 input 欄位）。
- **對外顯示用原創遊戲名**（這是原創機制，命名自取）。
- 無計時（回合制策略，慢思）。

### 規則速覽（前端顯示用，規則全在 server）

- 每人一座農場：5×3＝**15 格**，起始 2 房（木屋）+ 2 名家庭成員（工人）。
- **14 輪**，每輪揭示一個新行動格；玩家依序輪流「放 1 個工人到空行動格 → 執行動作」，工人放完進下一輪。
- **6 個收穫輪**（第 4/7/9/11/13/14 輪末）：田收成 → 餵食（每名成員 2 糧，不足以資源/牲畜煮食補，再不足記乞討）→ 牲畜繁殖。
- 第 14 輪結束 → 結算分數。

---

### 上行（Client → Server）

#### 大廳 / 房間
| type | data | 說明 |
|------|------|------|
| `join_lobby` | – | 訂閱大廳，回 `room_list` |
| `list_rooms` | – | 取房列表 |
| `create_room` | `{ room_name?, nickname? }` | 建房，自己當 host。`room_name`=房名(≤40)、`nickname`=暱稱(≤20) |
| `join_room` | `{ room_id, nickname? }` | 加入房 |
| `leave_room` | – | 離開（host 離開→解散；對局中→中止整局） |
| `start_game` | – | **僅 host**，2–4 人時開局 |

#### 對局
| type | data | 說明 |
|------|------|------|
| `action` | `{ action, input? }` | 在行動格放工人。`action`=行動字串（見下）；`input`=該動作參數 |

`action` 字串（18 種）：
`forest` `clay_pit` `reed` `quarry` `river` `sheep_pen` `boar_pen` `cattle_pen`（累積格：取走目前堆積量）、`grain_seeds`（+1 穀）`veg_seeds`（+1 菜）`plow`（+1 田）`sow`（播種）`fences`（圍牧場）`build_rooms`（蓋房+畜舍）`renovate`（升級房屋材質）`family_growth`（+1 成員）`day_labor`（+2 糧）`start_player`（搶先手+1 糧）。

`input` 欄位（只有對應動作才需要）：
```jsonc
{
  "grain_fields": 1,     // sow：以穀播種的（空）田數，每塊收 3 穀
  "veg_fields": 0,       // sow：以菜播種的田數，每塊收 2 菜
  "rooms": 1,            // build_rooms：蓋幾間房（每間 5 對應材料 + 2 蘆葦）
  "stables": 0,          // build_rooms：同時蓋幾座畜舍（每座 2 木）
  "pasture_tiles": 2,    // fences：牧場格數（花 格數+1 木，+畜舍再+2 木）
  "pasture_stable": false
}
```

### 下行（Server → Client）

| type | data | 對象 | 說明 |
|------|------|------|------|
| `room_list` | `{ rooms:[{id,name,players,max,status}] }` | 請求者 | |
| `lobby_update` | `{ rooms:[...] }` | 大廳訂閱者 | |
| `room_created` | `{ room_id }` | host | |
| `room_update` | `{ room_id, name, host_seat, players:[{seat,name}], can_start, your_seat }` | 房內（逐人） | `your_seat`=收訊者自己的 seat |
| `room_closed` | `{ reason }` | 房內其餘 | `host_left` / `aborted` |
| **`state`** | 完整盤面（見下） | 全房 | 開局時 + **每次動作後**廣播 |
| `game_over` | `{ scores:[int,...] }` | 全房 | 第 14 輪結算（index=seat），隨後房解散 |
| `error` | `{ reason }` | 觸發者 | |

#### `state` 盤面
> 逐人發送，每則帶收訊者自己的 `your_seat`（其餘欄位相同）。判斷輪到我：`current_player === your_seat`。
```jsonc
{
  "your_seat": 1,                  // 收訊者自己的 seat（逐人注入）
  "round": 3,
  "phase": "placing",              // 或 "game_over"
  "current_player": 1,             // 該放工人的 seat（null=結算中）
  "starting_player": 0,
  "available_actions": ["forest","plow", ...],   // 本輪仍可選的格
  "accumulation": [{ "action":"forest", "amount":6 }, ...], // 累積格現有量
  "players": [                     // index = seat
    {
      "house":"wood", "rooms":2, "family":2,
      "fields":[ {"crop":"grain","count":3}, null ],  // null=已犁未播
      "pastures":[ {"tiles":2,"stable":false,"animal":{"kind":"sheep","count":3}} ],
      "loose_stables":0, "free_tiles":11,
      "wood":1,"clay":0,"reed":2,"stone":0,"grain":0,"veg":0,
      "sheep":3,"boar":0,"cattle":0,"food":4,"begging":0
    }
  ]
}
```
> 完全資訊：所有玩家的農場都在 `state` 裡，前端直接整盤重繪即可（不必自己累積 delta）。

---

### JSON 範例
```jsonc
// 開局：host → start_game ；server → 全房 state
{ "game":"farm", "type":"start_game" }
// 輪到 current_player 的人 → action（犁田）
{ "game":"farm", "type":"action", "data":{ "action":"plow" } }
// 播種兩塊田（1 穀 1 菜）
{ "game":"farm", "type":"action", "data":{ "action":"sow", "input":{ "grain_fields":1, "veg_fields":1 } } }
// 取森林累積的木
{ "game":"farm", "type":"action", "data":{ "action":"forest" } }
// 結算
{ "game":"farm", "type":"game_over", "data":{ "scores":[12, 8, 15] } }
```

### reason 列舉
- `error.reason`（動作）：`bad_action` / `not_in_game` / `not_your_turn` / `locked`（格未揭示）/ `occupied`（格已被佔）/ `game_over` / `no_space`（農場滿）/ `no_materials` / `no_wood` / `no_seeds` / `not_enough_fields` / `no_room`（家庭成員＝房間數）/ `max_house`（已石屋）/ `bad_pasture`
- `error.reason`（房）：`already_committed` / `bad_room_id` / `room_not_found` / `room_full` / `already_started` / `not_in_room` / `not_host` / `cannot_start`
- `room_closed.reason`：`host_left` / `aborted`

### 計分（server 算，前端顯示用）
田/牧場/穀/菜/羊/豬/牛各有分級（0 個多為 −1，越多越高至 +4）；圈養畜舍 +1/座；土屋房間 +1、石屋 +2/間；家庭成員 +3/人；乞討 −3/個；農場空格 −1/格。`game_over.scores` 已是各人總分。

### 斷線
- 大廳：移除訂閱。
- 房內等待中：移除座位（host 斷線→解散，其餘收 `room_closed{host_left}`）。
- **對局中任一人斷線 → 整局中止**（`room_closed{aborted}`，房解散）。無重連。
