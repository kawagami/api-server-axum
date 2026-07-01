# blog-next

個人部落格與工具整合平台，Next.js 16 + React 19，後端對接 Rust Axum API (`axum.kawa.homes`)。

---

## 功能模組

前台路由加 locale prefix（`zh-TW` / `zh-CN` / `en`），`/` 自動 redirect 至 `/{defaultLocale}`。

| 路由 | 說明 |
|------|------|
| `/{locale}` | 首頁，文章列表 |
| `/{locale}/blogs/[id]` | 文章內容（Markdown 渲染） |
| `/{locale}/hackmd-notes` | HackMD 筆記整合，支援標籤篩選 |
| `/{locale}/tools/alarm` | 鬧鐘工具 |
| `/{locale}/tools/hourly-chime` | 整點報時（Web Speech API TTS） |
| `/{locale}/tools/convert-text` | 文字轉換工具 |
| `/{locale}/tools/countdown` | 倒數計時工具 |
| `/{locale}/tools/new-password` | 密碼產生工具 |
| `/{locale}/tools/roster` | 排班功能 |
| `/{locale}/games/chess` | 線上象棋，即時連線對戰（大廳開桌 / 快速配對，server 權威裁判） |
| `/{locale}/games/western-chess` | 線上西洋棋，即時連線對戰（易位／過路兵／升變，共用大廳框架） |
| `/{locale}/games/gomoku` | 線上五子棋，即時連線對戰（共用大廳框架） |
| `/{locale}/games/go` | 線上圍棋（19 路，貼目 7.5，雙虛手終局），共用大廳框架 |
| `/{locale}/games/banqi` | 線上暗棋（翻棋），即時連線對戰（共用大廳框架） |
| `/{locale}/games/avalon` | 線上阿瓦隆，5–10 人社交推理（私有角色／投票／任務／刺殺／房內聊天） |
| `/{locale}/games/farm` | 農場經營，2–4 人 worker-placement（完全資訊，每動作後整盤廣播） |
| `/{locale}/games/metal-slug` | 越南大戰，Bevy wasm 動作遊戲 |
| `/{locale}/ws` | WebSocket 即時訊息 |
| `/{locale}/dashboard` | 個人儀表板（需登入） |
| `/{locale}/dashboard/notifications` | 通知列表（需登入） |
| `/{locale}/portfolio` | 投資組合追蹤，記錄持股並查看每日漲跌盈虧（需登入） |
| `/{locale}/ledger` | 記帳，收支記錄＋分類/日期篩選＋統計總覽（收支結餘、分類圓餅、每月趨勢）；可掃電子發票 QR 順便記一筆支出（需登入） |
| `/{locale}/invoices` | 統一發票登錄＋自動對獎，掃 QR／掃條碼／手動登錄、我的發票／中獎、中獎 email 通知設定（需登入） |
| `/{locale}/lotto` | 大樂透／威力彩選號登錄＋自動對獎，手動選號或拍照 OCR 辨識（即時相機／拍照）、我的彩券／中獎、開獎結果、中獎 email 通知設定（需登入） |
| `/{locale}/profile` | 個人資料（需登入） |
| `/{locale}/about` | 關於頁面 |
| `/{locale}/login` | OAuth 登入（Google；GitHub / LINE 規劃中） |
| `/admin/*` | 後台管理（需登入，無 locale prefix）：文章、圖片、股票、會員、角色、使用者、WS、對局總覽、Torrents、日誌／稽核日誌、設定 |

---

## 技術棧

- **框架**：Next.js 16 (App Router, Turbopack)
- **UI**：React 19 + Tailwind CSS + lucide-react；主題系統 runtime 切換（森林/海洋/天空，admin 設定全站生效，CSS variables）
- **i18n**：next-intl v4，支援 zh-TW / zh-CN / en
- **Markdown**：react-markdown（rehype-highlight 程式碼高亮、rehype-slug 標題錨點 + 自動目錄 TOC）
- **圖片**：next/image（自動 WebP 轉換、lazy loading、縮圖），本地儲存（`/uploads/*`）
- **認證**：JWT（`jose`），middleware 保護 `/admin/*` 與 `/{locale}/dashboard|profile|settings|portfolio|ledger|invoices|lotto`
- **OCR**：`tesseract.js`（動態載入、數字白名單），彩券選號拍照／即時相機辨識輔助
- **後端 API**：`https://axum.kawa.homes`（Rust Axum）
- **WebSocket**：`wss://axum.kawa.homes`
- **部署**：Docker multi-stage build，standalone output

---

## 專案結構

```
app/
  [locale]/   # 前台（zh-TW / zh-CN / en）
  admin/      # 後台（無 locale prefix）
  api/        # Route Handlers
  auth/       # OAuth callback
components/   # 共用 UI 元件
i18n/         # next-intl 設定（routing / request / navigation）
messages/     # 翻譯字串（zh-TW.json / zh-CN.json / en.json）
api/          # 前端 API 請求函式（fetch 封裝，一個資源一檔：blogs / portfolio / images …）
types/        # 後端 API 共用型別
hooks/        # React custom hooks
libs/         # 工具函式庫
public/       # 靜態資源
```

---

## 本地開發

```bash
pnpm install
pnpm dev          # http://localhost:3000
pnpm lint         # ESLint（flat config）
```

> 套件管理用 pnpm（版本由 `package.json` 的 `packageManager` 釘住，`corepack enable` 即自動取用）。

需設定 `.env.local`：

```env
API_URL=https://axum.kawa.homes
WS_URL=wss://axum.kawa.homes
JWT_SECRET=...
```

---

## 部署

push `master` 即自動部署：GitHub Actions build + push image（`kawagami77/my-next-blog`）→ SSH 進 VPS pull + 重啟（`.github/workflows/cicd.yml`）。

- image 不含任何 env/secrets，設定由 VPS mount `.env.production` 於 runtime 注入
- multi-stage build（deps → builder → Node Alpine runner，non-root、standalone output）
- 本機驗證 image：`docker build -t blog-next . && docker run --env-file .env -p 3000:3000 blog-next`
