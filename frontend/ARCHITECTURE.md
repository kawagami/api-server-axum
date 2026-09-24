# kawa-web frontend — 架構與慣例

> 功能清單、本地開發與部署流程見同目錄 `README.md`；本檔是內部架構、慣例、踩過的坑與已知技術債。
> 對戰遊戲的 WS 協定在 monorepo 根 `protocol/games-wire.md`（前後端唯一契約）。
> 引用 `docs/**` 的連結指向**不進版控**的本機文件，只當備查；要長期成立的結論一律寫在本檔本文。

---

## 專案概述

個人部落格 + 工具整合平台。Next.js 16 App Router，後端為 Rust Axum（`https://api.kawa.homes`）。

---

## 架構慣例

### Server / Client 邊界

- **Server Component**：預設，不需任何指令。頁面（`page.tsx`）、layouts、`components/` 皆預設 server-side
- **Client Component**：需互動狀態時標 `"use client"`（hooks、事件處理、context 消費）
- **`"use server"`**：只用在 async function（Server Actions / `api/` 檔案），絕對不加在 component 或 page 檔案頂部
- 不要在 Server Component 裡用 `useState` / `useEffect`

### API 請求分層

```
api/                  ← fetch 封裝，一個資源一個檔案（blogs.ts / portfolio.ts / images.ts …），named exports
libs/adminRequest.ts  ← admin 操作（session cookie），401 → /admin/login。**server-only，不是 "use server"**（見下）
libs/memberRequest.ts ← member 操作（access_token cookie），401 → /{locale}/login（getLocale 取當前 locale）
libs/fetchApi.ts      ← 無認證 fetch 封裝（!res.ok 時丟 ApiError）
libs/api-error.ts     ← ApiError 型別 + apiErrorStatus / apiErrorMessage
libs/admin-route.ts   ← 後台讀取 Route Handler 的共用殼 adminJson()（server-only）
libs/admin-queries.ts ← 後台輪詢讀取的 client 端版本（打 app/api/admin/**，client-only）
types/index.ts        ← 後端 API 共用型別（Blog、User、Stock 等）
```

- **一個資源一檔**：同資源的 CRUD 全放同檔 named exports（如 `api/portfolio.ts` 匯出 `getPortfolioSummary` / `postPortfolio` / `putPortfolio` / `deletePortfolio`），不再一函式一檔
- 公開資料用 `fetchApi<T>()`（自帶錯誤處理），不手寫 raw fetch
- 需認證的操作（admin CRUD）用 `adminRequest<T>()`（泛型，回傳型別安全）
- member 自身資料（如 `getCurrentMember`）用 `memberRequest<T>()`
- 所有 api/ 檔案都是 `"use server"`（`scripts/check-conventions.sh` 逐檔檢查第一行）
- ⚠️ **`libs/adminRequest.ts` / `libs/memberRequest.ts` 是 `import "server-only"`，絕對不要改回 `"use server"`**（2026-09-25 修）。`"use server"` 會把 export 登記成 Server Action（有 action ID、server 會受理呼叫），而這兩支收的是**完整 URL** 並自動帶上 session token —— 當時 build 產物的 action manifest 裡確實有它們，只是 ID 沒進 client bundle 才打不到；哪天被 client 元件 import，就是免登入也能用的 SSRF 代理（打得到內網 `backend:3000` / database 並把回應帶回去）。`server-only` 讓誤 import 直接 build 失敗；conventions 腳本也擋 `libs/` / `components/` / `hooks/` 出現 `"use server"`
- **後台「會被輪詢或連續觸發」的讀取走 Route Handler，不走 Server Action**（2026-09-25）。Next 在 client 端**一次只送一個 Server Action**（後一個等前一個跑完，Next 文件 server-actions「Sequential dispatch」），輪詢與使用者操作（載入更多、展開軌跡、刪除…）因此互相排隊。現在 `app/api/admin/{games,ws/connections,metrics,stats/visitors,logs,audit-logs}/route.ts` 各包一支 `api/*.ts` 的讀取函式（`libs/admin-route.ts` 的 `adminJson`：`unstable_rethrow` 讓 adminRequest 的 401 redirect 照常變成 307、其餘錯誤沿用後端狀態碼 + `errorData`；參數白名單、列舉值不合法就當沒給），client 端 `libs/admin-queries.ts` 提供**同名同簽名**的函式（`fetch(..., { redirect: "manual" })`，307 時自己導去 `/admin/login?redirect=`，不白抓登入頁 HTML），6 個輪詢元件只換 import 來源。**不做通用代理**（那等於把 adminRequest 開成 endpoint）；寫入照舊走 Server Action。仍走 Server Action 的讀取：其他 `usePagedList` 消費點、前台 vocab / 持股歷史 / 文章留言 —— 沒有輪詢，排隊的影響小，碰到再搬。2026-09-25 本機實測：6 支 route 帶 session 皆 200、未登入 307、session 被後端撤銷 307 並保留 redirect 路徑、後端稽核確實以 admin 身分記到 route 發出的呼叫
- **`api/github.ts` 是唯一手寫 `fetch(` 的 api 檔**：它打的是 GitHub 公開 API 而非自家後端，套不上 `fetchApi`（那支拼 `API_URL`、丟 `ApiError`）。仍守 `"use server"` + named exports；失敗一律回空陣列（`/changelog` 顯示空狀態），不讓 GitHub 掛掉變成整頁 500
- **分頁端點的 api 函式一律回整包 `PaginatedResponse<T>`，不在 api 層解包成陣列**（2026-08-03 統一，2026-08-07 隨後端補齊 total 擴到全部端點）。解包等於替所有呼叫端丟掉 `total`。消費端直接把函式當 fetcher 餵給 `usePagedList`（`load(page => getXxx({ page, per_page: LIMIT }))`），`total` 由 hook 回傳。`adminRequest` / `memberRequest` 可能回 null，故一律 `return res ?? { data: [], total: 0 }`（範本 `api/admin-vocab.ts`）

### 環境變數

- `process.env.API_URL`：後端 base URL（server-side only；production 是 docker 內網位址 `http://backend:3000`，compose service 名，見 `deploy/docker-compose.yml`）
- `process.env.API_PUBLIC_URL`：後端**公開** base URL（`https://api.kawa.homes`），拼給瀏覽器直接打的連結用（如 torrent 下載連結）；沒設 fallback `API_URL`（本地開發兩者相同）
- `process.env.WS_URL`：WebSocket URL（server-side 讀，經 WsProvider 的 `wsUrl` prop 傳給 client；**不用 NEXT_PUBLIC_**，避免 build 時烤進 bundle）
- `process.env.JWT_SECRET`：JWT 驗證用，不暴露前端
- `process.env.GITHUB_REPO`：`/changelog` 的資料來源 repo（`owner/name`）。**沒設時 fallback `kawagami/api-server-axum`**（本站自己的 repo，本地與 production 零設定就能用）；設成**空字串 = 關閉這頁**（`/changelog` 404、`/about` 不顯示入口、sitemap 不收錄）—— 商家 instance 共用同一份 image，不該顯示 kawa 的 commit 紀錄
- 三者皆 runtime 注入（docker-compose `env_file`），image 內不烤 .env

### 認證

- **身份中文命名**：`user`（後台 RBAC 帳號，有 roles/permissions）面向使用者一律稱 **「管理員」**；`member`（OAuth 前台使用者）一律稱 **「會員」**；`role` 稱「角色」。程式碼變數、API 路徑、cookie 名稱維持英文不動，命名共識只套用在中文文案
- Admin JWT 存在 `session` cookie；member OAuth token 存在 `access_token` cookie
- **Admin passkey 登入（WebAuthn）**：密碼登入的可選升級（密碼永遠保留）。登入頁（`app/admin/login/page.tsx`）三入口——Conditional UI（name 欄 `autoComplete="username webauthn"`，掛載即發起 discoverable 挑戰，autofill 選 passkey 即登入）、「使用 Passkey 登入」按鈕（modal 模式）、密碼登入成功後的升級提示卡（無 passkey 且未略過時顯示；略過寫 `localStorage.passkey_prompt_dismissed_at`，30 天內不再問）。登入前挑戰走同源代理 `app/api/auth/passkey/login/{begin,finish}/route.ts`（finish 成功寫 session cookie，比照 login）；登入後的註冊/列表/刪除走 `api/auth.ts` server actions（`beginPasskeyRegistration`/`finishPasskeyRegistration`/`getPasskeys`/`deletePasskey`）。管理頁 `/admin/passkeys`（nav「設定」群）。ceremony 用 `@simplewebauthn/browser`，`optionsJSON` 一律傳後端回傳物件的 **`.publicKey` 內層**；`AbortError`/`NotAllowedError` 是取消不是錯誤（StrictMode double-effect 也會觸發）要吞掉
- `proxy.ts` 保護 `/admin/*` 與 `/{locale}/` 下的 `dashboard`、`profile`、`portfolio`（`memberPaths` 共 3 條。**沒有獨立的 `/settings`** —— 功能的設定子頁由所屬 prefix 涵蓋，不另列）；admin 未登入 → `/admin/login`，member 未登入 → `/{locale}/login`
- 401 回應由 `adminRequest` / `memberRequest` 統一處理並 redirect 到對應 login
- member 登入會保留目標頁：proxy 導向時帶 `?redirect=<原路徑>`，login 頁轉給 `/api/auth/{provider}` 寫入短效 httpOnly cookie `post_login_redirect`，OAuth callback 讀取後回跳（含 open-redirect 防護：只收站內相對路徑）。**新增 OAuth provider 按鈕時，onClick 須比照 google 傳 `redirectTo`**

### i18n（next-intl v4）

- 支援語言：`zh-TW`（預設）、`zh-CN`、`en`
- 前台路由加 locale prefix：`/{locale}/blogs/[id]`、`/{locale}/login` 等
- Admin 路由（`/admin/*`）、API（`/api/*`）、OAuth（`/auth/*`）**不加** locale prefix
- 翻譯字串放 `messages/*.json`，namespace 對應元件（`Header`、`Login` 等）
- Server Component 用 `getTranslations('Namespace')`；Client Component 用 `useTranslations('Namespace')`
- 公開頁的 `Link` / `redirect` / `useRouter` 從 `@/i18n/navigation` 引入（自動帶 locale），不用 `next/link`
- Admin 元件繼續用 `next/link`（不需要 locale 感知）
- 新增翻譯：在 `messages/zh-TW.json`、`messages/zh-CN.json`、`messages/en.json` 同步加 key（縮排 2 空白）
- **前台不留寫死的中文**：`/tools` 四頁（`Timers` 一頁三張卡＝`Alarm` / `Countdown` / `HourlyChime` 三個 namespace、`ConvertText`、`NewPassword`、`Roster`）與錯誤頁（`Error` / `NotFound`）都已有 namespace。**server action 不要回傳中文訊息**（那裡拿不到使用者語系），回 `messageKey` 讓 client 翻（見 `tools/convert-text/actions.ts`）。整點報時的語音內容與 `SpeechSynthesisUtterance.lang` 也跟著語系走（`HourlyChime.speech` / `speechLang`）
- **不要把後端錯誤訊息印給使用者**：後端 `errors.rs` / `services/*.rs` 的訊息全是寫死繁中，`setError(e.errorData?.message || t('errorSave'))` 這種寫法等於 en / zh-CN 使用者看到中文。一律用 `libs/api-error.ts` 的 `apiErrorStatus(e)` 依 status 分流到自己 namespace 的 key（409=已存在、422=`errorInvalid`、其餘 `errorSave`）。後端回應的 `code` 只是 HTTP 狀態碼、不是機器可讀原因，別拿來做更細的分支
- **後端回的資料標籤同理**：後端回的中文常數標籤不要直接渲染（en / zh-CN 使用者會看到中文），一律用 value 當 i18n key、後端 label 只當未知 key 的 fallback
- `app/layout.tsx` 的 metadata 走 `generateMetadata` + `getTranslations('Home')`：**不要寫死 description**（沒自帶 description 的頁面會繼承它，寫死等於三語系都吃到中文）。`WsProvider` 掛在 `NextIntlClientProvider` 之外拿不到 `useTranslations`，斷線橫幅文案由 root layout 以 `lostLabel` prop 餵進去（同 `ThemeButton` 的 `labels`）
- `app/layout.tsx` 的 `<html lang>` 用 `getLocale()`，**不要寫死 `zh-TW`**；locale 之下的 404 走 `(public)/not-found.tsx`（有 Header/Footer 與翻譯），`app/not-found.tsx` 只是 locale 之外的最後防線（仍走 `getTranslations('NotFound')`，不寫死中文，只是取不到 locale 時落在 defaultLocale）
- 前台日期顯示：用 `Intl.DateTimeFormat(locale, { …, timeZone: 'Asia/Taipei' })`，locale 跟隨當前語系、時區固定台北，**不要**寫死 `toLocaleString('zh-TW')`（server/client 格式一致、免 `suppressHydrationWarning`）。blog 卡片共用 `components/blogs/show-client-time.tsx`

### 全域狀態

- 深色模式走 `globals.css` 的 `@custom-variant dark (&:where(.dark, .dark *))`（Tailwind v4 的寫法，等同 v3 的 `darkMode: 'selector'`），`.dark` 掛在 `<html>` 上。優先序：使用者 `theme` cookie ＞ admin 設定 `default_color_mode`（light/dark/system）＞ 系統 `prefers-color-scheme`；`libs/color-mode.ts` 管 resolve/套用，ThemeButton 三態循環（亮→暗→跟隨預設=清 cookie）

---

## 樣式慣例

> 樣式全部集中在 `app/globals.css`（Tailwind v4 起沒有 `tailwind.config.js`）。

- **主題系統：runtime 切換、全站設定**。色階走 CSS variables（`app/globals.css` 定義 `:root`=forest 與 `[data-theme="ocean|sky|sunset|sakura|grape|mono"]` 共 7 套），同檔 `@theme` 的 `--color-primary-*` / `--color-neutral-*` 各 11 階以 `rgb(var(--primary-500))` 引用那些 var（**不要**把色值直接寫進 `@theme`，那會讓 runtime 換主題失效）。opacity modifier（`bg-primary-900/30`）由 v4 用 `color-mix(in oklab, …)` 產生，不再需要 v3 的 `<alpha-value>` 佔位。主題值存後端 settings（key `site_theme`），root layout 經 `getPublicSettings()`（`api/settings.ts`，60s cache）設 `<html data-theme>`，admin settings 頁 ThemePicker 改值（`updateSiteTheme` action + `revalidatePath('/', 'layout')`）；`libs/site-theme.ts` 管常數/resolve
- 元件用色一律 `primary-*`（品牌色）與 `neutral-*`（中性灰），**不用** `gray` / `stone` / `blue` / `indigo` 等具體色名
- 語意色保留：紅=錯誤、`libs/badge-styles.ts` 的 HTTP method / log level 色、股票紅漲綠跌（台股慣例）、**橘／琥珀 = 警示或未知值**（`say-something-form` / `webauthn-settings` 的提醒、`tools/roster/shift-badge.tsx` 的班別與 default 班別）、**遊戲資產色**（棋盤木色 `bg-amber-*`、farm 農作物）。這些不算違規，換成 `primary-*` 會失去語意
- body 背景：`bg-gradient-to-b from-primary-50 to-neutral-100`（dark: `from-primary-950 to-neutral-900`），背景粒子特效在 `components/theme-background.tsx`（forest=落葉、ocean=氣泡、sky=雲朵橫飄、sunset=餘燼、sakura=花瓣、grape=紫氣泡、mono=灰雲，色走 var，含 `prefers-reduced-motion` 關閉）
- Logo：`components/kawa-logo.tsx`（inline SVG 吃 var 跟主題變色）；favicon `app/icon.svg` 固定 forest 色
- Transition：**禁止全域 `* { transition: all }`**，互動元素個別掛 `transition-colors` / `transition-shadow`
- Hover scale：只用在塊級卡片/按鈕（上限 `hover:scale-105`），文字連結用變色 + underline，不縮放
- Loading：統一 `Loader2`（lucide）spin + `animate-pulse` skeleton，不再有自訂 CSS loader。**前台資料頁一律要有自己的 `loading.tsx`**，用 `components/loading/public-page-skeleton.tsx` 的 `<PublicPageSkeleton width nav variant rows />`（variant：`list` / `cards` / `form`），參數要對齊該頁的 `PageShell`
- **一次性提示**：用 `components/toast.tsx` 的 `useToast()` + `<Toast toast={toast} />`（固定底部置中、`role="status"`）。**前後台通用** —— 該檔只吃 message 字串、不碰 i18n，所以沒有 `NextIntlClientProvider` 的 admin 也能直接用（`tag-manager` / `blog-action-buttons` 就是）。**不要用 `alert()`**，也不要各頁自刻覆蓋層。破壞性操作的 `confirm()` 確認保留（刪除持股）。分工：清單列的行內動作回饋（塞不進塊狀元素）與成功訊息走 Toast，頁面層級的「載入／操作失敗」走 `ErrorBanner`
- 換頁淡入由 `components/page-transition.tsx` 統一提供（掛在 `(public)/layout.tsx`），**不要只給單一區塊做轉場**
- Tailwind CSS，深色模式用 `dark:` prefix
- RWD 斷點：`sm:` 開始展開，行動優先
- **前台捲動模型（單一來源）**：捲動一律由 **body** 負責，header 是 `sticky top-0`（`components/header.tsx`，半透明底 + `backdrop-blur`）。頁面**不要**自己開 `h-[calc(100svh-120px)] overflow-auto` 的內捲區——那會讓 header 捲走、與其他頁不一致，也讓子層 `sticky` 失效；`(public)/layout.tsx` 的 `<main>` 同理**不能**有 `overflow-hidden`。唯一例外是**遊戲**（棋盤要塞滿一屏不捲），`games/**` 維持自己的固定高度。錨點偏移由 `globals.css` 的 `html { scroll-padding-top: 66px }` 統一處理（66 = header 50 + 呼吸 16），側欄 sticky 也用 `top-[66px]`
- **前台版面寬度與 padding（單一來源）**：一律用 `components/page-shell.tsx` 的 `<PageShell width>`（`mx-auto w-full max-w-* px-4 py-6 sm:py-8`）。三種寬度：`form`=表單／設定／單欄詳情（`max-w-2xl`）、`content`=一般內容頁（`max-w-4xl`，預設）、`wide`=卡片網格 hub（`max-w-5xl`）。**頁面不要再自己寫 `max-w-*` / `px-4` / `py-8`**。頁內間距用 `PageShell className="flex flex-col gap-6"`，子元件不要自帶 `mb-6`。`loading.tsx` 的容器規格要跟 `page.tsx` 一致，否則載入完會跳版
- **前台頁面標題**：一頁一個 `components/page-title.tsx` 的 `<PageTitle title description actions variant />`。只有兩種字級：`default`（靠左 `text-2xl`，內容頁／會員頁）與 `hero`（置中 `text-3xl sm:text-4xl`，首頁與 tools / games / about hub）。**不要再各頁自己寫 `text-4xl font-extrabold` 這類一次性標題**
- **對話框**：置中彈窗一律用 `components/modal.tsx` 的 `<Modal label onClose dismissible size surface backdrop className />`（2026-08-30 收斂，取代 9 處手寫殼）。它包掉背景遮罩、`role="dialog"` / `aria-modal`、以及 `hooks/useDialog.ts` 的 Esc 關閉／焦點鎖在框內／背景捲動鎖／關閉還原焦點。`dismissible={false}` 用在請求進行中（避免以為取消了但已送出）。`className` 只放內距／捲動／排版，**底色圓角陰影寬度由元件給、不要覆寫**（`cn` 不做 Tailwind 衝突合併）。`surface` 分 `admin`（`dark:bg-neutral-900`）與 `public`（`dark:bg-neutral-800`），因為前後台卡片底色本來就差一階。**不收非置中的浮層**：抽屜（admin sidebar / logs trace）、header 手機選單、命令面板、遊戲結局遮罩，硬塞只會讓 props 長成另一套 CSS
- **className 組合**：條件式接字串用 `libs/cn.ts` 的 `cn()`（3 行、無依賴）。刻意**不裝** clsx / tailwind-merge —— 全庫用法都只是「基礎樣式 + 幾段條件樣式」。⚠️ `cn` **不處理 Tailwind 衝突覆寫**（`px-2` 與 `px-4` 同時出現時贏的是 CSS 產生順序），要覆寫就把該屬性從基礎樣式拿掉，不要疊上去賭順序
- **輸入欄位樣式**：`libs/input-styles.ts` 是單一來源（2026-08-30 收斂 14 份各自宣告的 `inputClass`）。三個常數：`ADMIN_INPUT`（後台表單欄位，不含 `w-full`）、`ADMIN_FILTER_INPUT`（後台篩選列的窄欄位）、`PUBLIC_INPUT`（前台**卡片內**的欄位）。尺寸差異（`w-full` / `text-sm` / `font-mono`）用 `cn()` 疊，不再開新常數。唯一例外是 `contact/contact-form.tsx` —— 它不在卡片上，底色要跟著頁面漸層走
- 後台版面高度用 `h-screen`（無 header/footer）；**後台登入頁不要沿用前台扣 120px 的公式**（沒有 header/footer，用 `min-h-screen` + 置中）
- 圖示庫：`lucide-react`
- Markdown 排版：`@tailwindcss/typography` 已安裝，用 `prose prose-stone dark:prose-invert` class。閱讀頁 `components/blogs/blog-article.tsx`（`ReactMarkdown` + `remark-gfm`）；程式碼高亮用 `rehype-highlight` + `highlight.js` 的 `github-dark` 主題（配 prose 深色 `pre` 底，亮/暗模式皆一致），標題錨點用 `rehype-slug`，右側 TOC 用 `libs/blog-markdown.ts` 的 `extractHeadings`（github-slugger，slug 與 rehype-slug 一致）。⚠️ `highlight.js` 必須列**直接依賴**，否則 `import 'highlight.js/styles/...css'` 在 pnpm 嚴格 node_modules 下解析不到、build 失敗
- **Admin 版面寬度與 padding（單一來源）**：外層 padding 與內容寬度都由 `app/admin/(main)/layout.tsx` 給（`px-4 pb-4 pt-3 sm:px-6…` + `mx-auto w-full max-w-6xl`）。**頁面不要再自己加 `p-3 sm:p-6` / `max-w-*` / `mx-auto` / 灰底**（會疊 padding 並讓各頁寬度不一，切頁時卡片左右跳）。只有兩種版型：表格／儀表板／圖表頁＝全寬（吃 layout 容器）；表單／詳情頁＝最外層 `max-w-2xl`（靠左，不置中）。`loading.tsx` 的容器規格要跟 `page.tsx` 對齊，否則載入完會跳版
- **Admin 頁面標題**：一頁一個 `components/admin/page-header.tsx` 的 `<PageHeader title description actions />`（`h1` 規格、右側動作區），頁面主體包在 `flex flex-col gap-4` 裡。**統計數字不要用 `h1` 冒充標題**（用 `dl` / 純文字）
- **前台 metadata / SEO**：`app/layout.tsx` 設 `title: { template: '%s｜Kawa's Homes', default: … }`，各頁 `metadata.title` 只填自己的短名（站名不要再寫進 `metaTitle`，會重複兩次；首頁例外，用 `title: { absolute }`）。多語系頁面的 canonical + hreflang 一律用 `libs/seo.ts` 的 `localeAlternates(locale, path)`，**不要手寫 `alternates: { canonical }`**（少了 `languages` 三語系會被當重複內容）。`app/sitemap.ts`（依 `enabled_features` 過濾、文章清單抓不到時靜默略過，不讓 build 掛掉）、`app/robots.ts`、`app/opengraph-image.tsx`（預設分享圖，**只能用拉丁字母**，ImageResponse 預設字型沒有中日韓字）。`proxy.ts` 的 matcher 要排除 `opengraph-image`（沒副檔名會被 intl middleware 加 locale prefix 而 404）
- **Admin metadata**：`app/admin/layout.tsx` 已設 `title: { template: '%s｜後台', default: '後台' }`，各頁 `metadata.title` 只填中文短名。**client component 的 page 不能 export metadata**，放同目錄的 `layout.tsx`（login / passkeys / change-password 就是這樣）
- Admin 表格：用 `components/admin/table.tsx` 的 `AdminTable` / `AdminHeadRow` / `AdminRow` / `AdminTh` / `AdminTd` / `AdminEmptyRow`，不要重複手寫 cell class，**也不要另做一套「只有底線」的表格**（兩套視覺會讓頁面看起來不像同一個後台）。整列底色（如 log level）用 `AdminRow` 的 `tone` prop 取代預設 hover，別把兩個 `hover:bg-*` 疊在一起。文字色掛在 `<table>` 靠繼承，個別 cell 才能用 `text-neutral-500` 這類覆寫。清單空了一律 `AdminEmptyRow`，不要只剩表頭空殼
- **Admin RWD**：多欄寬表格一律包一層 `overflow-x-auto`（手機橫向捲、不裁切）；放在卡片 `overflow-hidden` 內時 wrapper 要在卡片**內側**，共用容器 `components/admin/admin-table-container.tsx` 已是此設定。sidebar 已內建漢堡選單 + drawer（`admin-sidebar.tsx`），新增頁面不必再處理導航 RWD
- **Admin 長清單的黏住表頭**：wrapper 改成 `admin-sticky-head overflow-auto max-h-[70svh]`（`AdminTableContainer` 用 `stickyHead` prop），CSS 規則在 `globals.css`。**兩個前提缺一不可**：sticky 要下在 `<th>` 上（下在 `thead`／`tr` 上不會生效），且捲動容器要有高度上限——單純 `overflow-x-auto` 的 wrapper 高度等於內容，而 `overflow-x: auto` 會讓 `overflow-y` 計算成 `auto`，sticky 於是無處可黏（舊 stocks 頁 `thead` 上的 `sticky top-0` 就是這樣從未生效）。代價是表格有自己的捲動區（頁面外層仍可捲），短清單不必套
- **會折行的內容不要用 `inline-flex`**（表格 cell 尤其容易中）：`inline-flex` 是 atomic inline-level 盒子，只以**第一行**的基線參與外層行框計算，內容折到第二行以後就溢出行框、疊到下面的元素上（gov_tenders 的標案名稱連結 + 類別說明實際踩過）。圖示 + 可能折行的文字要嘛走純 inline 流（圖示用 `inline-block` + `align-[-2px]`，會自然跟在最後一行後面），要嘛整個改 `flex`（block-level，高度算得對）。只有內容保證不折行時（`whitespace-nowrap` 的 cell、按鈕文案）`inline-flex` 才安全
- **Admin 表格的欄寬三種踩法（2026-08-11 逐頁修過一輪，新表格照這條檢查）**：`AdminTable` 預設是 auto layout，欄寬由各欄的 min-content／max-content 決定，以下三種都會讓「最該讀的欄變最窄」或把最後一欄推出容器。①**某欄 `break-all`**：min-content 直接掉到 1 字寬，等於告訴瀏覽器「我不需要空間」，而鄰欄的長 token（`api_server_axum::utils::reqwest`、query string）不可斷、反過來吃走全部寬度 → 該欄改 `wrap-break-word`（**不是** `break-all`，v4 正名，別寫 `break-words`），並給長 token 欄 `w-*` + `truncate` + `title`（logs / audit_logs / ws 都是這型）。②**只有一兩欄設了 `min-w`/`max-w`，其餘放給 auto**：視窗一窄（側欄收起、瀏覽器放大）就撐過容器 → 整張表 `table-fixed` + 逐欄 `w-*`，只留一個「吃剩餘空間」的主欄不設寬（gov_tenders）。③**欄數多**：cell 預設 `px-4`＝單欄 32px，11 欄光 padding 就 352px，自然寬度貼齊容器，垂直捲軸一出現（-15px）最後一欄就被切掉 → 加 `admin-table-dense`（globals.css，padding 減半），**不要**用 `table-fixed`（數字欄被截斷等於看不到值）。判斷方式：欄數 × 32 + 各欄實際內容寬 是否逼近 `max-w-6xl`。④**`table-fixed` 的後遺症：`whitespace-nowrap` + 放不下的欄寬 = 文字溢出格子、疊到右邊那欄**（auto layout 會依 nowrap 的 min-content 自動給足寬度，fixed 不會）。
- **欄寬不要寫死 px**。px 等於把字寬心算編進 class，換字型 / 字級 / 語系就得回頭重算，而算錯就是破版（實際連踩三次：gov_tenders 類型欄、logs 與 audit_logs 時間欄、audit_logs 的 `DELETE` badge）。改用內容相對單位讓瀏覽器量：**不該折行的欄**（日期、代號、badge）用 globals.css 的具名工具 `col-datetime` / `col-date` / `col-id` / `col-badge`（都是 `calc(Nch + 2rem)`，`ch` = 字型裡「0」的實寬，`2rem` 補回 cell 的 `px-4`，因為 box-sizing 是 border-box）；**會折行的欄**用 `w-[Ne m]`（全形 CJK 一字約 1em），猜小了只是多折一行、不會破版。新增這類欄位就加一條 `col-*`，不要回頭寫 px。
- **`whitespace-nowrap` 只在 auto layout 下是安全的**（瀏覽器會給足寬度）。`table-fixed` 表裡只留給真的不能斷行的欄（日期），其餘一律讓它折行 —— nowrap 等於宣告「這欄寬度不由我控制」。
- **結構性保險**：`table.tsx` 的 cell 帶 `overflow-hidden`，所以欄寬估錯最多是裁切，不會再疊到隔壁欄。cell 的 `px-4 py-2` 比焦點框（2px outline + 2px offset）寬，格子內按鈕的焦點框不會被裁。已套 `table-fixed`：logs / audit_logs / gov_tenders / messages / blogs；已套 `admin-table-dense`：stock-day-all / vocab
- **Admin 表格吃剩餘高度（`AdminTableContainer` 的 `fill` prop）**：`max-h-[70svh]` 是猜的高度上限，內容只要比一屏高一點點，admin layout 的 `overflow-auto` 就會跟表格自己的捲動區各長一條垂直捲軸、兩條並排。`fill` 把上限換成「吃掉版面剩下的高度」，整頁剛好一屏、只有表格會捲。**前提是整條高度鏈每一層都要 `flex min-h-0 flex-1 flex-col`**：`(main)/layout.tsx` 的 `max-w-6xl` 容器已是 `h-full` flex 欄（其餘頁面的 root 是普通 flex item，`min-height:auto` 不會被壓縮，內容過長照樣把外層撐出捲軸，所以不影響沒用 `fill` 的頁），呼叫端 page root → 表格外層 → `AdminTableContainer fill`。**少一層 `min-h-0` 就會被內容撐開、退回兩條捲軸**。另一個代價：清單很短時卡片仍是滿高的空白。**全部長清單頁都已套用，而且都走 `AdminTableContainer`**（logs / audit_logs / gov_tenders / vocab / messages / blogs / blog-comments / torrents / members / stocks 的 list・stock-day-all・get-buyback-plans・get-unfinished-buyback-price-gap）。⚠️ 其中 6 頁（audit_logs / gov_tenders / vocab / blog-comments / torrents / stocks 的 list）原本是**手抄同一組 class**（`admin-sticky-head overflow-auto min-h-0 flex-1` 配一個自寫白卡 div），效果一樣但改元件影響不到它們 —— 2026-08-30 已全部收斂進元件，`admin-sticky-head` 現在只出現在 `globals.css` 與 `admin-table-container.tsx`，可以用 `grep -rn admin-sticky-head app` 守住這件事。載入中淡化（`transition-opacity` + `isPending`）一律包在容器**外**一層 `flex min-h-0 flex-1 flex-col`，元件本身不吃這個狀態。`stickyHead` 單獨使用（`max-h-[70svh]`）已無呼叫端、只留作短清單的選項。新的長清單頁一律 `fill`，不要再回頭用 70svh
- **Admin 側欄底部**：`SidebarFooter`（`admin-sidebar.tsx` 內）＝身分（name + 超級管理員/管理員）＋深淺色切換（與前台共用 `ThemeButton` 與同一個 `theme` cookie，props 由 `(main)/layout.tsx` 算好傳入）＋快速跳頁＋回前台＋登出。身分來自 `GET /admin/auth/me` 的 `name` / `is_super_admin`（`is_super_admin` **只用於顯示**，權限判斷一律看 `permissions`）
- **Admin 快速跳頁（⌘K / Ctrl+K）**：`components/admin/command-palette.tsx`，選單來源同樣是 `adminNavGroups`（由 `AdminSidebar` 傳入**已依權限與功能開關過濾**的 groups），所以新增頁面不必動這個元件。比對標籤／分組名／路徑三者任一。快捷鍵監聽在 `AdminSidebar`（palette 關閉時不掛載，聽不到自己的快捷鍵）
- **Admin 清單篩選一律進 URL**：用 `hooks/useFilterUrl.ts`（`{ initial, write }`）—— 初始值從 query 讀、變更用 `history.replaceState` 寫回。**不要用 `router.replace`**（清單已在 client 端抓好，再跑一次 server render 是白工，還會動捲軸）；replace 而非 push 是刻意的，避免每改一個下拉就堆一筆「上一頁」。已套用：logs / audit_logs / gov_tenders / vocab / blogs（torrents、stocks 走 server searchParams，本來就在 URL 上）。讀 `useSearchParams` 的 client 元件，page 要用 `<Suspense>` 包住。`audit_logs` 的 `from`/`to` 在 URL 上是 **ISO**（與 `/admin/metrics` 選區產生的連結同一格式），輸入框用 datetime-local，兩邊各自轉換
- **Admin 時間顯示**：一律用 `libs/admin-datetime.ts`（`formatDateTime` / `formatDateTimeSeconds` / `formatDate` / `formatTimeOfDay`，釘死 `zh-TW` + `Asia/Taipei`）。**禁止 `new Date(x).toLocaleString()`**——server component 跑在容器裡（沒設 TZ＝UTC）會少 8 小時，client 又跟著瀏覽器跑，同一份資料在不同頁長得不一樣。要特殊格式（只有月日、只有時分）就用該檔匯出的 `ADMIN_LOCALE` / `ADMIN_TIME_ZONE` 自建 formatter
- **焦點樣式（全站）**：`globals.css` 有一條全域 `:is(a,button,input,select,textarea,summary,[tabindex]):focus-visible` 的 outline 規則（原本只套 `.admin-shell`，已提為全站，前後台一致）。**元件不要自己寫 `focus:ring-*` / `focus:outline-none` / `focus:border-*`**（逐一補一定會漏，且會和統一規則疊成雙層框）。前台曾漂移出 36 處自寫版本（含 4 份逐字相同的 `inputClass`、4 種 ring 粗細），2026-07-31 已全數移除；`canvas` 之類非原生互動元素只要有 `tabIndex` 就吃得到這條規則，不必自己補。前台另有 skip link（`.skip-link`，`(public)/layout.tsx` 的第一個可聚焦元素）與行動選單的 focus trap（`header.tsx`，關閉時把焦點還給漢堡鈕）
- **Admin modal / drawer**：用 `hooks/useDialog.ts`（Esc 關閉、鎖背景捲動、焦點鎖在對話框內、關閉後還原焦點），容器自行補 `role="dialog" aria-modal="true"`。常駐 DOM 靠 transform 滑出的 drawer，關閉時要加 `inert`，否則鍵盤仍可 Tab 進看不見的內容
- **Admin 錯誤呈現**：頁面層級（清單載入失敗、操作失敗）用 `components/admin/error-banner.tsx` 的 `<ErrorBanner>`（含 `LOAD_FAILED` / `DELETE_FAILED` 常數，帶 `role="alert"`）；欄位層級的驗證訊息留在欄位旁的 inline `<p>`。**不要用 `window.alert`**
- **Admin 骨架屏**：每個 route 都要有 `loading.tsx`，形狀對齊實際頁面。共用 `components/loading/table-skeleton.tsx`（`BorderedTableSkeleton` / `ListTableSkeleton`）、`form-skeleton.tsx`、`chart-page-skeleton.tsx`
- **後台文案一律繁中**（含 nav 標籤、表頭、按鈕、空狀態、登入頁）；`Email` / `Query` 這種技術名詞可保留。後端回的英文狀態值（torrent / stock status）用 `libs/badge-styles.ts` 的 `*_STATUS_LABEL` 對照表轉中文，查不到則顯示原字串
- Admin 導航：側邊欄（`components/admin/admin-sidebar.tsx`）與首頁 quick links（`app/admin/(main)/page.tsx`）共用 `components/admin/nav.ts` 的 `adminNavGroups` 單一來源，改選單只動這檔。麵包屑（`admin-breadcrumb.tsx`）也是反查這份 nav——**沒登記在 nav 的路徑會 fallback 顯示原始英文路徑段**，所以新頁面要嘛登記進 nav，要嘛別做成使用者到得了的獨立頁（股票原本的 `/admin/stocks` hub 頁就是因此移除，nav 直接列 4 個子頁）
- **`/admin/metrics` 圖表選區查操作紀錄**：`metrics-trend-chart.tsx` 支援橫向拖曳選 `TimeRange`（端點取採樣點的 `created_at` ISO，四張圖共用同一組 range state 故可用字串比對定位灰帶；單擊只看數值不動選區），選好由 `metrics-audit-panel.tsx` 打同一支 `getAuditLogs({from,to})` 就地列出。需 `metric:read` + `audit:read` 兩個權限（page 用 `getMyPermissions()` 判斷後傳 `canReadAudit`，沒有就不啟用拖曳）。面板的「在操作紀錄開啟」帶 `?from=&to=`（ISO），`audit-logs-client.tsx` 首次渲染讀 `useSearchParams` 轉成 datetime-local 當初始條件（故 page 需 `<Suspense>` 包住）
- 前台導航單一來源：`libs/site-nav.ts` 的 `TOOLS` / `GAMES` / `MEMBER_LINKS`（含 lucide icon）。header 工具/遊戲下拉與 `/tools`、`/games` index 頁共用前兩者；header 會員下拉與 `/dashboard`「我的功能」共用 `MEMBER_LINKS`。新增項目＝site-nav 加一行 + `Header` labelKey + 對應 hub namespace（`ToolsHub`/`GamesHub`/`Dashboard`）的 `items.{key}` 描述（三語系同步）。**instance 功能開關**：後端 `enabled_features` 設定（`"all"` 或 JSON 字串陣列，`GET /settings/public` 下發），`libs/enabled-features.ts` 的 `resolveEnabledFeatures`（null = 全開）+ `isFeatureEnabled` 收斂；key 權威在後端 `Feature` enum，`BACKEND_FEATURES` 只給後台 picker 列選項（兩邊同步加）。過濾點：Header/tools/games/dashboard 走 `site-nav.ts` 的 `filterNavByFeatures`（item 的 `feature` 欄，省略 = 核心不受控）、首頁卡片 = `home_features` ∩ `enabled_features`（registry 的 `feature` 欄）、後台側欄與 quick links 走 `filterNavByPermissions` 第三參數。管理 UI 是**專頁 `/admin/platform`**（`enabled-features-picker.tsx`，portfolio 依賴 stocks 會連動；另有 `webauthn-settings.tsx` 管 passkey 的 `webauthn_rp_id`/`webauthn_rp_origin`——配對規則「rp_id 是 origin 本身或其上層網域」的**權威在後端**（`validate_webauthn_pair`），兩個 key 走批次端點 `PATCH /admin/settings` body `{ values: {...} }` 一次原子寫入（同 transaction、全過才寫），所以後端能無條件驗最終狀態而不會讓整組換網域死鎖；前端同一份檢查只是即時回饋，不是唯一防線。nav 標 `permission: "platform:read"`，商家管理員無此權限整個選單項不存在，後端 GET 也濾掉保留 key、PATCH 需 `platform:update`）。與 `home_features` 職責不同：前者 = 站有沒有這功能（影響 API 404），後者 = 首頁展示與排序（純展示）。功能卡片共用 `components/feature-card.tsx`（首頁 / index 頁 / dashboard 同款）；**首頁卡片清單走 `libs/home-features.ts` 的 `HOME_FEATURES` registry**（icon/href/後台中文 label 單一來源），顯示與排序由後端設定 `home_features`（JSON 字串陣列，`GET /settings/public` 下發）控制，`resolveHomeFeatures` 收斂（未知 key 忽略、缺值/壞值 fallback 全顯示、空陣列＝全隱藏），admin settings 頁 `home-features-picker.tsx` 管理（`updateHomeFeatures` action + `revalidatePath('/', 'layout')`）。新增首頁卡片＝registry 加一行 + `Home.features.{key}` 三語系；header 的「工具」「遊戲」文字是連到 index 頁的 Link，chevron 才是展開下拉（桌面另支援 hover 展開）。dashboard 的「快速操作」deep-link 區塊已隨發票/樂透移除而拿掉。**首頁（`project-intro.tsx`）除卡片網格外還有「最新文章」區塊**（`latest-posts.tsx`，抓 `getBlogs` 最新 3 篇，受 `blog` 功能開關控制、包在 `<Suspense>` 內讓 hero 先出來、後端掛掉 catch 成空陣列後整段不渲染，文案在 `Home.latest.*`）；卡片網格欄數由 `gridClass(count)` 決定（1 張置中窄欄、2 或 4 張走 2 欄、其餘 3 欄），避免 instance 關功能後卡片被拉成超寬或出現 3+1 孤兒列
- **`/changelog` 更新紀錄頁直接讀 GitHub commits**（`api/github.ts` → Next Data Cache `revalidate: 3600`、tag `changelog`），刻意不經後端：只是唸公開資料，為它在 backend 開 endpoint + Redis 快取不划算。要搬到後端的時機是「instance 變多」、「repo 轉私有需要 token」或「想在後台人工編修」
- **顯示哪些 commit 的規則在 `libs/changelog.ts`**（`CHANGELOG_TYPES` = feat / fix / perf / refactor / security，`scope === "deps"` 也濾）。歷史上 chore 47 筆、docs 21 筆，全列會把真正的功能更新洗掉；抓 100 筆濾完顯示上限 40 筆（實測 100 → 62 筆合格）。commit message 是繁中，三語系都會看到繁中內容 —— 這是刻意接受的取捨（要三語就得改成手寫 changelog）
- **type chip 不用語意色**：紅/綠/橘在本站有既定意義（見「語意色保留」），commit type 一律 primary chip + lucide icon 區分（`page.tsx` 的 `TYPE_ICON`）
- ⚠️ **`notFound()` 在 `[locale]/(public)` 下回的是 HTTP 200 + 404 頁面內容**（Next 16.3.1 實測，`/contact` 的 feature gate 與 `/changelog` 的 `GITHUB_REPO` gate 都一樣；真正不存在的路徑才是 404）。等於 soft 404，SEO 上不理想但目前全站一致，要改就一起改
- **定時輪詢一律用 `hooks/usePolling.ts`**（`usePolling(cb, intervalMs, enabled?)`），**不要自己寫 `setInterval` 打 API**；輪詢的讀取函式從 `libs/admin-queries.ts` 取（理由見「API 請求分層」）。它在 `document.hidden` 時跳過該次請求、回到前景且已過一個週期才補一次 —— 瀏覽器只會把背景分頁的 timer 節流到 ≥1 分鐘，**不會**停掉 fetch，管理員把後台分頁擱在背景就會一直打後端（1 核 1G 不划算）。已套用：`metrics-view` / `visitor-stats-view` / `games-overview` / `ws-connections` / `audit-logs-client` / `logs-client`。`cb` 每次 render 取最新（存 ref），呼叫端不必 memo；純顯示用的 tick（如 `ws-connections` 的連線時長、`_shared/Clock`）不打 API，維持原本的 `setInterval`
- 「載入更多」分頁清單：用 `hooks/usePagedList.ts`（`usePagedList<T>()` 或 `usePagedList<T>({ items, total, fetcher })` seed 第 1 頁；`load(fetcher)` 重設、`loadMore()` 下一頁）。**fetcher 回整包 `PaginatedResponse<T>`，不要在呼叫端解包**——`hasMore` 是拿「已載入筆數 < total」算的（精確），`total` 也一併回傳可直接顯示「共 N 筆」。2026-08-07 前後端所有分頁端點統一回 `{ data, total }`，在那之前有 6 支回裸陣列，全站只能猜「這頁滿了就假設還有下一頁」，最後一頁剛好滿 per_page 時會多出一顆按不出東西的按鈕。另有請求序號閘：每次 `load`/`loadMore` 取遞增序號，回應時序號對不上就整包丟棄（fetcher 是 Server Action、無法 abort，只能在回應端裁決），所以連按「搜尋／重設」或快速改篩選條件時，慢回的舊查詢不會蓋掉新結果；`loadMore` 的頁碼在發請求時就推進、失敗才退回。抓取失敗回 `failed` 布林（**不回訊息**——fetcher 是 Server Action，production 下 Next 會把拋出的 error 抹成通用訊息只留 digest，拿不到後端 `code`/`message`；401 的 `NEXT_REDIRECT` 已濾掉不算失敗）。文案由呼叫端出：後台 `<ErrorBanner message={failed ? LOAD_FAILED : null} />`（`components/admin/error-banner.tsx` 匯出常數），公開頁用各 namespace 的 `loadFailed` key
- Markdown 編輯器行為（Tab/Shift+Tab 縮排、Enter list 接續、貼 URL 包成 `[選取](url)`、貼圖即時上傳）：用 `hooks/useMarkdownTextarea.ts`（回傳 `{ ref, handlers, insert }`，spread `handlers` 到 textarea），編輯邏輯是 `libs/markdown-edit.ts` 的純函式（`indent`/`unindent`/`continueList`/`wrapLink`/`insert`，零 DOM、可測）；縮排字元改 `markdown-edit.ts` 的 `INDENT` 常數

---

## 檔案結構

```
app/
  layout.tsx          # root：html/body/WsProvider only，無 Header/Footer
  [locale]/           # i18n route segment（zh-TW / zh-CN / en）
    layout.tsx        # NextIntlClientProvider，驗證 locale 合法性
    (public)/         # 前台 route group（Header + Footer）
      layout.tsx
      [route]/
        page.tsx
        layout.tsx    # 有需要時加
        actions.ts    # Server Actions
  admin/              # 後台，完全獨立，不繼承前台 Header/Footer，無 locale prefix
    layout.tsx        # admin shell
    login/
    (main)/           # 需登入的後台頁
      layout.tsx      # sidebar layout
      [route]/
        page.tsx
        actions.ts
  api/                # Next.js Route Handlers（不在 route group 內，無 locale）
  auth/               # OAuth callback（無 locale）
components/
  [feature]/          # 被多個頁面共用的功能性元件放子目錄
  loading/            # 共用 skeleton（BorderedTableSkeleton / ListTableSkeleton），loading.tsx 引用
  modal.tsx           # 置中對話框外殼（背景遮罩 + a11y），見「樣式慣例」
api/
  {resource}.ts       # 一個資源一檔（blogs / portfolio / images / members / users / roles / logs / tools / ws / auth / github（外部 API，見上））
i18n/
  routing.ts          # locales 定義（zh-TW / zh-CN / en）、defaultLocale
  request.ts          # getRequestConfig，載入對應 messages
  navigation.ts       # locale-aware Link / redirect / useRouter（公開頁用，取代 next/link）
messages/
  zh-TW.json          # 繁體中文翻譯
  zh-CN.json          # 簡體中文翻譯
  en.json             # 英文翻譯
types/
  {resource}.ts       # 後端 API 共用型別，一資源一檔（portfolio / torrent …）
  index.ts            # barrel re-export，消費端一律 import 自 @/types
hooks/          # React custom hooks（.ts/.tsx）
libs/           # 工具函式庫（.ts）
```

> `postcss.config.js` 維持 `.js`（內容只剩 `@tailwindcss/postcss` 一個 plugin，v4 自帶 prefix 所以沒有 autoprefixer），Next.js 設定檔不需遷移。`tailwind.config.js` 已隨 v4 移除。

### 檔名與擺放（2026-07-31 統一）

- **檔名一律 kebab-case**（`blog-action-buttons.tsx`）。例外只有兩類：hook 檔用 camelCase（`hooks/usePagedList.ts`、`components/blogs/useBlogDraft.ts`），以及 **`games/**` 維持 PascalCase**（自成子系統、內部一致，改名會擴散到 `_shared` 全部 import）。元件識別字本身仍是 PascalCase，只有檔名走 kebab。
- **只被單一 route 用的元件 colocate 在該 route 目錄**，用相對路徑 `./xxx` import（`tools/timers/`、`tools/roster/`、`portfolio/`、`games/metal-slug/`、`contact/`、`vocab/` 都是）。被多頁共用的才留 `components/`。
- **metadata 擺放看 page 是不是 client component**：server page → `page.tsx` 直接 export；client page → 同目錄 `layout.tsx`（`tools/convert-text`、`new-password`、`roster` 的 pass-through layout 是必要 workaround，不是重複，別想合併）。
- **例外：admin-only 元件仍留在 `components/`**（`roles/`、`images/`、`stocks/`、`blogs/` 的編輯器那幾支）。`components/blogs/` 混著前台用的 `blog-article` / `blog-list` / `show-client-time`，要拆得先逐支判歸屬，本輪刻意沒動。
- **Server Actions 的擺放沒有統一規範**（2026-08-30 盤點）：頂層 `actions/` 只有 `auth.ts` 一支，其餘 8 支都是 colocate 在 route 下的 `actions.ts`。`actions/auth.ts` 是被多個 route 共用才提上去的 —— 新增時照這條判斷：**單一 route 用就 colocate，跨 route 共用才放頂層 `actions/`**；純資料存取一律進 `api/{resource}.ts`，不要開新的頂層 action 檔。
- **`components/[feature]/` 子目錄至少要有兩支檔案**才成立（規則是「被多頁共用」）。目前 `components/roles/`（1 支）與 `components/blogs/comments/`（1 支）是無謂分層，各自只有一個消費點，攤平成 `components/roles-manager.tsx` 即可 —— 2026-08-30 記下，未動。

### 工具鏈

- **套件管理用 pnpm**（版本由 `package.json` 的 `packageManager` 釘住，`corepack enable` 自動取用）。lockfile 是 `pnpm-lock.yaml`，無 `package-lock.json`
- Lint：`pnpm lint`（`eslint .`，flat config `eslint.config.mjs`）。ESLint 固定 v9 — eslint-config-next 內附 plugin 尚不相容 v10。**2026-09-25 起 CI 會跑**（在那之前 CI 只跑 tsc，lint 規則沒人守），error 擋、warning 不擋
- **專案慣例由 lint 守**（`eslint.config.mjs`）：公開頁（`app/[locale]/**` 與 `components/` 頂層）禁 `next/link` 與 `next/navigation` 的 `useRouter` / `redirect` / `usePathname`（改用 `@/i18n/navigation`；外部連結以 `eslint-disable-next-line` 註明，目前 3 處都是 GitHub 連結）；`api/**`、`actions/**`、`app/**/actions.ts` 禁 `revalidateTag`（改 `updateTag`）；全站禁 `alert()` 與不帶參數的 `new Date(x).toLocale*String()`
- **ESLint 管不到的在 `scripts/check-conventions.sh`**（CI 同樣跑）：`"use server"` 只能在 `app/` / `api/` / `actions/`、`api/*.ts` 第一行必須是 `"use server"`、打後端一律走 `api/*.ts`（例外：fetch 封裝本身與要寫 cookie 的認證 Route Handler）、`@/types/` 子路徑 import、`gray`/`slate`/`zinc`/`indigo` 色名、`focus:ring-*` 等自寫焦點樣式、`admin-sticky-head` 只能在 `AdminTableContainer`。要破例就改腳本並寫明理由
- Type check：`pnpm exec tsc --noEmit`
- pnpm 10 預設擋原生套件 build script；sharp 用 prebuilt binary 不受影響。日後加需編譯的原生套件要 `pnpm approve-builds` 或在 package.json 設 `pnpm.onlyBuiltDependencies`
- `reactStrictMode: true`（dev 期 double-render 抓副作用）

---

## 後端 API

base URL：`process.env.API_URL`（`https://api.kawa.homes`，舊名 `axum.kawa.homes` 仍為有效 alias）

常用端點（參考 `api/` 目錄的實際實作）：
- `GET /blogs/` — 文章列表（分頁），params: `page`(預設1)、`per_page`(預設10)、`tag`(可選)，回傳 `{ data: Blog[], total }`（後端 `Paginated<T>` 只有這兩欄，**不回 `page` / `per_page`**）
- `GET /blogs/tags` — 所有 tags 字串陣列（去重、字母排序），無需認證
- `GET /blogs/tags/counts` — 每個 tag 附文章數（`TagCount[]`），無需認證；公開列表側欄的 tag 篩選用（`api/blogs.ts` 的 `getBlogTagCounts`，與 `getBlogTags` 同吃 `tags:['blogs']` 快取標籤）
- `GET /blogs/:id` — 單篇文章
- `PUT /admin/blogs/:id` — 新建或更新（upsert），需 `blog:update` 權限，body: `{ markdown, tags }`，回傳 204 無 body
- `DELETE /admin/blogs/:id` — 刪除，需 `blog:delete` 權限，回傳 204 無 body
- `GET /admin/auth/me` — 目前登入管理員，回 `{ id, name, permissions, is_super_admin }`（`libs/admin-permissions.ts` 的 `getCurrentAdmin` 以 `cache()` 去重，同一次請求只打一次）
- `POST /admin/auth/refresh` — admin JWT 續期，`Authorization: Bearer <token>`，no body
- **Admin passkey**：`POST /admin/auth/passkeys/register/begin`（需認證，回 CreationChallengeResponse）、`POST /admin/auth/passkeys/register/finish` body `{ label, credential }`（201；409=已註冊過、422=label 超長）、`POST /admin/auth/passkeys/login/begin`（公開，回 `{ auth_id, options }`）、`POST /admin/auth/passkeys/login/finish` body `{ auth_id, credential }`（回 JWT，與 `POST /admin/auth` 同形；失敗一律 401，挑戰 5 分鐘過期）、`GET /admin/auth/passkeys`（`PasskeyItem[]`）、`DELETE /admin/auth/passkeys/:id`（204；非本人 404）
- `GET /oauth/{provider}` — OAuth 登入 URL 取得
- `POST /oauth/{provider}/exchange` — OAuth code 換 token，body: `{ code, state }`
- `POST /admin/users` — 建立使用者，回傳 201 無 body
- `POST /roster` — 排班計算。body `{ names, days, rule, morning_slots?, night_slots?, max_consecutive? }`（`rule` 是後端 enum，未知值 422；slots 兩者同給或同省），回 `{ status, data, plan, warnings }`。**`warnings` 是機器碼**（`understaffed` / `shift_uncovered` / `night_to_morning` / `max_consecutive_exceeded`），文案在 `Roster` namespace 的 `warn*` key（`tools/roster/page.tsx` 的 `WARNING_KEYS`）。純函式與上限鏡射在 `libs/roster.ts`（`MAX_NAMES` / `MAX_NAME_LEN` / `MAX_DAYS` 要與 `backend/src/structs/roster.rs` 同步）
- `GET /members` — 會員列表，需認證（`member:read` permission），回 `{ data, total }`
- `GET /members/:id` — 會員詳細 + OAuth providers，需認證
- `GET /ws/connections` — 線上連線列表，需認證（`ws:read` permission）。**2026-07-31 從 `get_online_connections` 改名**（動詞路徑 → 資源路徑）
- `POST /ws/messages` — 對指定連線推送訊息，body: `{ addr, message }`，需認證（`ws:read` permission）。**同日從 `say_something_to_someone` 改名**
- `POST /ws/ticket` — 換發 WS 一次性連線票（30 秒 TTL），需 admin 認證，回傳 `{ ticket }`；前端經同源 `/api/auth/ws-ticket` 代打
- `GET /admin/stocks/changes` — 股票異動列表，分頁 `page`/`per_page`（預設 50），需認證
- `PATCH /admin/stocks/changes/:id/pending` — 更新單筆 pending，id 在 path、無 body，回傳 204，需認證
- `GET /admin/stocks/day_all` — 每日行情，分頁 `page`/`per_page`（預設 100），需認證
- `GET /admin/stocks/buyback_price_gaps` — 未完成庫買價差，需認證
- `GET /admin/stocks/buyback_periods` — 庫買期間，需認證
- `GET /admin/audit_logs` — 分頁 `page`/`per_page`（預設 100），回 `{ data: AuditLog[], total }`（2026-08-07 起，原為裸陣列）；query 支援 `user_email` / `method` / `path` / `from` / `to` / `actor_type`（`admin` / `member`，不給 = 兩者都列；member 的稽核 2026-08-09 才開始記，且**只記非 GET**）
- `GET /logs` — 分頁 `page`/`per_page`（預設 100），回 `{ data: Log[], total }`（2026-08-03 起，原為裸陣列）；query 另支援 `level`（逗號分隔多值、大小寫不敏感）/ `q`（message 與 fields 模糊）/ `target` / `request_id` / `from` / `to`，**前端目前用 `level` + `q`**（兩者都進 URL，走 `useFilterUrl`）。`Log` 另有 `request_id` 與 `fields`（`fields.self` = 真正的錯誤細節，`message` 只是固定字串），**`logs-client.tsx` 的展開面板已渲染這兩欄**（`self` 固定排第一）
- `GET /logs/request/:request_id` — 單一請求的完整 log 軌跡（時間正序、不分頁、回裸陣列），需 `log:read`；前端已接（`api/logs.ts` 的 `getLogTrace`，每列一顆「整條軌跡」鈕就地展開）
- `GET /admin/gov_tenders` — 政府採購網標案列表（需 `gov_tender:read`），query `keyword`/`tender_type`（完全比對）/`q`（標案名稱/機關模糊）/`page`/`per_page`（預設 50），回傳 `{ data: GovTender[], total }`；`api/gov-tenders.ts` 的 `getGovTenders` 回整包 `PaginatedResponse<GovTender>`（同上方「api 層不解包」規則），消費端在 `usePagedList` 的 fetcher 內用。`GET /admin/gov_tenders/types` 回所有出現過的類型（`string[]`，去重排序），頁面篩選下拉（`getGovTenderTypes`）用。頁面 `/admin/gov_tenders`，`detail_url` 為官方公告頁外連。資料由後端排程每日抓取，前端唯讀
- `GET /admin/settings` — 設定（依 category 群組），需認證；`PATCH /admin/settings/:key` body `{ value }`，`site_theme` 接受 7 套主題（forest/ocean/sky/sunset/sakura/grape/mono）+ `auto`（每日輪播），非法回 422
- `GET /settings/public` — 公開設定白名單（無認證，後端記憶體 map 直讀）。**值已是該有的型別**（2026-08-07 起）：`theme_rotation` 是物件、`home_features` / `enabled_features` 是陣列（`enabled_features` 為 `"all"` 時仍是字串）、`image_client_compress` 是布林、`image_client_quality` / `image_client_max_edge` 是數字。原本整包回字串（JSON 字串包在 JSON 裡），每個消費端都得自己再 parse 一次。**後端轉不動的值會原樣退回字串**，故各 resolver 仍要吃得下字串形（`resolveEnabledFeatures` 就吃兩種）。`api/settings.ts` 的 `PublicSettings` 欄位型別刻意寫 `unknown`：值是 runtime 可改的設定，收斂責任在各 resolver
- 分頁參數全站統一 `page`（從 1 起算）/`per_page`（上限 200）
- `POST /admin/roles`、`POST /member/portfolio` 回傳 201 + entity
- `GET /member/portfolio` — 當前 member 的投資組合列表，需 access_token 認證，回傳 `PortfolioEntry[]`
- `POST /member/portfolio` — 新增持股，body: `{ stock_code, buy_date, cost_per_share, shares }`，回傳 `PortfolioEntry`
- `PUT /member/portfolio/:id` — 更新持股（只能更新自己的），body 同 POST，回傳 `PortfolioEntry`
- `DELETE /member/portfolio/:id` — 刪除持股（只能刪自己的），回傳 204
- `GET /member/portfolio/summary` — 所有持股 + 即時市價 / 股票名稱 / 盈虧 + **各期間增減**，回傳 `PortfolioSummaryEntry[]`；重用 history 邏輯取最新一筆，同 stock_code 共用 Redis cache。`changes` = `{ day, week, month }`，每個是 `PeriodChange`（`base_date` / `base_close` / `change` / `change_pct` / `value_change`）或 `null`（新持股、行情沒補齊、或基準日太舊 —— 後端寧可回 null 也不給名不副實的數字）。基準價已還原期間內除權息，**前端不要自己算**。⚠️ **端點沒有 `period` 參數**：一次回三組，切換是純前端（`portfolio/period-tabs.tsx`，總覽用今日/近一週/近一月、歷史表格用近一週/近一月/全部）
- `GET /member/portfolio/:id/history` — 持股每日歷史價格，含除權息還原成本，回傳 `HistoryRecord[]`（`{ date, close, adjusted_cost, pnl, pnl_pct }`）；後端打 TWSE + Redis 快取

**Blog 注意事項**：
- `html` 欄位已從資料庫移除（2025-01-05），不需送
- `GET /blogs/` 回傳分頁結構，`data[].tocs` 為 `string[]`（純文字標題陣列）；`total` 為過濾後總筆數
- **`tocs` 由後端從 markdown 解析**（`services/blogs.rs` 的 `extract_toc_texts`，2026-08-07 起），body 不再收 `tocs` —— 原本編輯器自己 parse 一份送上去，等於讓 client 決定文章標題（`tocs[0]` 會進列表卡片 / WS 廣播 / SEO title）。後端那份規則對齊 `libs/blog-markdown.ts` 的 `extractHeadings`（剝 code fence、去行尾 closing `#`），比舊的前端版更正確
- `PUT /admin/blogs/:id` 是 upsert，UUID 不存在時自動新建
- **快取**：`api/blogs.ts` 的 `getBlogs`/`getBlog`/`getBlogTags` 走 Next Data Cache（`next: { revalidate, tags: ['blogs', \`blog:${id}\`] }`），**不用 `no-store`**（public layout 讀 `cookies()` 強制動態渲染、SSG 用不上，只能靠 fetch data cache 去重後端請求）。`putBlog`/`deleteBlog` 寫入後 `updateTag('blogs')` + `updateTag(\`blog:${id}\`)` 即時失效。⚠️ **Server Action 的寫入路徑一律用 `updateTag(tag)`，不要用 `revalidateTag(tag, 'max')`**：Next 16 的 `revalidateTag` 帶非 `expire:0` 的 profile 時會刻意不設 `pathWasRevalidated`（避免 action 讀到自己的寫入），連帶**不失效瀏覽器 Router Cache**，症狀是存檔後重新進編輯頁看到舊內容、只有 hard reload 才更新（2026-07-26 實際踩過）。`revalidateTag` 只留給 Route Handler / 非 action 情境（那裡不能用 `updateTag`，會 throw）

### 圖片系統

舊 `/firebase` 路由已棄用（原代理至 FastAPI + Firebase）。舊的多檔端點 `POST /admin/images/upload_multiple` 已移除，改為單檔（前端本就一張一請求，見下方「圖片上傳行為」）。

現行系統（本地儲存）：
- `POST /admin/images` — 單檔上傳，multipart 單一 `file` 欄位，回傳 `201` + `{ id, url }`，需 Bearer token
- `GET /admin/images` — 列表，需認證，回傳 `[{ id, storage_key, url, status }]`（`status`: `active` / `unused`，後端用 cron job 清除 `unused` 圖片）
- `DELETE /admin/images/:id` — 刪除，需認證，回傳 204 No Content
- 圖片公開網域一律為 `media.kawa.homes`（nginx 直出磁碟）；URL base 由後端 `app_settings.upload_base_url` 決定（現值 `https://media.kawa.homes`，程式 fallback 同）。**存量舊圖已於 2026-07-28 全數回填成 media 網域**（後端 migration `20260728000000_media_domain_backfill`），DB 內不再有 `axum.kawa.homes/uploads/...`

前端對應：`api/images.ts`（單一整合檔，named exports `getImages` / `uploadImage` / `deleteImage` —— 上傳是**單數**，一張一請求，多張由 `libs/client-image.ts` 的 `compressAndUploadEach` 逐張呼叫）

**圖片顯示**：一律用 `next/image`（`import Image from 'next/image'`），不用原生 `<img>`。自動 WebP 轉換、lazy loading、縮圖。`remotePatterns` 只有 `media.kawa.homes/**`（舊網域 entry 已隨 2026-07-28 回填移除；`next/image` 只驗初始 URL 的 hostname，殘留舊 URL 會被擋成 400 而不是跟隨 301，所以 DB 必須先乾淨）。ReactMarkdown 內覆寫 `img` renderer 套用 `next/image`。例外：`blob:` 預覽與外部 OAuth avatar URL 用原生 `<img>` + eslint-disable（無法經 next/image 最佳化）。

**圖片上傳行為**：貼上（單張）或點按鈕選擇（支援多張）後立即上傳（immediate），一張一請求（`libs/client-image.ts` 的 `compressAndUploadEach`：上傳第 i 張時同步壓縮第 i+1 張）。後端負責管理圖片生命週期（status 欄位 + cron job），前端不追蹤 blob URL。

**上傳前壓縮可設定**：前端上傳前的縮圖/轉 WebP 由後端設定控制（`GET /settings/public` 下發 `image_client_compress` / `image_client_quality` / `image_client_max_edge`），`libs/image-config.ts` 的 `resolveImageCompressConfig` 收斂成 `ImageCompressConfig`（壞值 fallback 預設 on/q80/2560）。**交付走 server-prop**：admin server page（`blogs/[id]/page.tsx`、`images/page.tsx`）`getPublicSettings()` 後把 `compressConfig` 當 prop 傳給 client 元件（**刻意不走 `getSettings()` admin API**，避免 blog 編輯者需要 `setting:read` 權限的耦合）。`compress=false` 時跳過壓縮直接送原檔，後端 `process_image` 照樣 decode 驗證+轉檔。後端另有 `image_webp_quality`（僅後端讀、不公開）控重編碼品質。管理走 `/admin/settings` 的「儲存」分組（`field-config.ts` 已登記型別）。

**錯誤形狀**：`adminRequest` / `memberRequest` 用 `response.text()` 統一讀 body，空字串回 `null`，parse 失敗也回 `null`，正確處理 204 空 body。失敗時丟 `Error`（message: `API {status}: {statusText}`），並附 `.status`（number）與 `.errorData`。**`fetchApi` 2026-08-07 起也附這兩個欄位**，型別與取值 helper 收在 `libs/api-error.ts`（`ApiError` / `apiErrorStatus` / `apiErrorMessage`）——在那之前 `fetchApi` 不帶 status，`contact/actions.ts` 只能用 `msg.includes("429")` 比對錯誤訊息字串判狀態碼（改文案就靜默失效）。**新的 catch 區塊用 helper，不要再 inline cast**（既有 7 處 `err as Error & { status?: number; ... }` 尚未收斂（2026-09-25 重數仍是 7），改到時順手換掉；數字與下方「已知的技術債」那條同一份，改一邊要同步）。

---

## 部署

流程細節見 `README.md`「部署」一節（進版控，是唯一可信來源）。動手時要記住：

- **push `master` = 直接上 production**（CI 的 `build` 與 `test` 並行、閘門在 `deploy: needs: [test, build]` —— **test 不過不會部署，但 image 仍會推上 Docker Hub**）
- **env 全部 runtime 注入**，image 不含任何設定值；build 階段不需要 env（fetch 失敗有 fallback + 10s timeout）
- 本目錄**沒有** `docker-compose.yml`；部署編排只有 monorepo 根 `deploy/docker-compose.yml` 一份。本地驗證 image 用 `docker run`（見 README「部署」）

---

## WS 架構

- 全站用 `libs/ws-context.tsx` 的 `WsProvider`（layout 注入），建立單一 WS 連線
- **admin 身分連線走一次性 ticket**：root layout 只傳 `hasSession` 布林（token 不進 RSC payload），client 連線前打同源 `POST /api/auth/ws-ticket`（server 端用 session cookie 向後端 `POST /ws/ticket` 換 30 秒一次性票），再以 `?ticket=` 連 WS；票是一次性的，**每次重連都換新票**，換票失敗退回匿名連線。JWT 不出現在 WS URL / access log
- 後端 `user_joined` / `user_left` 事件（含 `real_ip`/`user_email`）**只推給 admin 連線**，匿名訪客收不到
- 訊息格式：`{ type: WsEventType, data: unknown }`。`types/ws.ts` 的 `WsEventType` 與後端 `structs/ws.rs` 的 `WsEvent` enum **一一對應**（新增事件兩邊同步加）；`WsNotifyEventType` 是「會彈 toast / 進通知列表」的子集（排除 `torrent_*`，那些只有後台 torrents 頁在看、每秒推一次）
- 訂閱用 `useWsContext()` 的 `subscribe(type, fn)` / `unsubscribe(type, fn)`（listener 第二參數拿到整則 `WsMessage`，含 `game` 欄供分流）；上行用 `send(type, data?, game?)`（連線未開時暫存，onopen flush；`game` 為對戰遊戲框架信封欄，一般 WS 訊息省略）
- `app/[locale]/(public)/dashboard/notifications/` — notification feed 頁（`components/ws/notification-feed.tsx`），訂閱所有 event type 即時顯示，入口在 header 會員下拉／dashboard。**原本另有一支無導航入口的 `/ws` 孤兒頁 render 同一個元件，且未被 `proxy.ts` 保護，已於 2026-07-31 刪除** —— 要再開 debug 頁記得同時加進 `memberPaths`
- 全站 toast 是 `components/ws/ws-toast.tsx`（掛在 `(public)/layout.tsx`，每個語系都會彈，文案走 `Ws` namespace，**不要寫死中文**）
- `app/admin/(main)/ws/` — 後台管理頁，查看線上連線 + 對指定連線送訊息
- `app/[locale]/(public)/games/` — **對戰遊戲平台**（象棋 `chess`、西洋棋 `western_chess`、五子棋 `gomoku`、圍棋 `go`、暗棋 `banqi`），複用 /ws 端點。大廳/桌位/配對/計時/斷線全遊戲共用：`_shared/`（`useRoomBase` WS 訂閱/join_lobby/錯誤映射/重連/unmount 離場底座（三個 room hook 共用）、`useGameRoom` 房間狀態機、`GameFrame` 外框含 `extraControls`/`extraStatus` 槽、`Lobby`、`Clock`、`useBoardCursor` 鍵盤游標、`pointer` 座標換算、`sound`、`wire` 型別）；各遊戲只寫 `XxxBoard`（盤面渲染 + 走步意圖）＋ `XxxGame`（接 useGameRoom）＋ `xxx-logic`（純資料模型）。

  **2026-08-24 這一輪補的（五個盤面共用，改盤面前先讀）**：
  - **鍵盤與螢幕閱讀器**：`_shared/useBoardCursor.ts` 提供 roving tabindex（整個盤面只有一個 tab 停留點＝游標所在格，方向鍵移動、Enter/Space 落子），命中層元素帶 `role="button"` + `aria-label`（座標＋棋子字面／`●○`／`?`／`×`，語言中立所以不進 i18n），svg 是 `role="group" aria-label={t('title')}`。方向鍵走**視覺方向**（`flipped` 由我方顏色決定）。⚠️ **命中層必須每格都渲染，包含已有子/已佔用的格** —— 那同時是游標的落腳處，跳過的話方向鍵移過去 focus 不到，游標與焦點分家後之後的方向鍵全失效（落子本身由 `play()` / `activate()` 自己擋）。
  - **合法步提示**：`room.hints`（server 推的 `hints` 訊息，見 `backend/ARCHITECTURE.md`「對戰遊戲框架」與 `protocol/games-wire.md`「目前狀態」）。象棋／西洋棋／暗棋畫選中子的合法目標（空格小圓點、可吃子畫圈），圍棋畫禁著點紅叉並在前端先擋（**判定仍在 server**），五子棋沒有。前端**不自己算規則**這條沒有變 —— 提示是 server 給的資料。
  - **拖曳落子**：滑鼠／觸控筆按下己方子進拖曳（`_shared/pointer.ts` 的 `toViewBox` 把 clientX/Y 換回 viewBox 座標，各盤面自己寫 `unproject`），放開處落子，過程中畫跟著指標的子與落點框。**觸控不進拖曳**（手指按住會與頁面捲動打架），觸控維持點選→點目標。
  - **觸控二次確認**：圍棋／五子棋的落子與暗棋的**翻子**在觸控裝置上先畫半透明預覽 + 琥珀色提示環，再點同一點才送出（`isTouchPointer` 看 `pointerType`，接滑鼠的二合一裝置不受影響）。理由：手指命中面積比格子大，而這幾個動作不可撤回。
  - **時鐘**：`_shared/Clock.tsx` 改吃 `baseMs` + `baseAt`（收到 server 時鐘的本地時間），每 tick 用 `Date.now()` **重算**而非累減 —— 背景分頁的 timer 被節流到 ≥1 秒，累減會把少扣的時間永久留在畫面上。重設也不再靠父層 `key` 重新掛載。⚠️ 計算放在 effect 裡：render 期呼叫 `Date.now()` 會被 `react-hooks/purity` 判為不純（`useGameRoom` 的 WS handler 同理，包了一層模組層 `nowMs()`）。
  - **走步被拒有話說**：`room.moveError`（i18n key）顯示在狀態列（`aria-live="polite"`），4 秒後自動消失。server 早就回 `illegal_move.reason`，原本前端只抖一下丟掉。reason → key 走 `useGameRoom` 的 `ILLEGAL_KEYS` 白名單（同 `KNOWN_ERR` 的理由：引擎新增 reason code 不該在畫面上變成 next-intl 缺 key 訊息），文案在 `GameLobby.illegal_*` / `illegalGeneric`。
  - **pending 有逾時**：送出 move 後 6 秒沒有任何回覆就解鎖盤面並提示 `moveTimeout`。原本封包掉了就永久停在 `pending=true`，盤面鎖死且畫面沒有任何說明，只能重整。
  - **結束畫面**：勝負／原因之外加了手數（`movesCount`）、耗時（`duration`）與最後幾手（`lastMoves`，`useGameRoom` 收 `formatMove` 回傳的短記譜，保留最後 6 手）。
  - 新增的共用檔：`_shared/pointer.ts`（座標換算 + 觸控判定）、`_shared/useBoardCursor.ts`（鍵盤游標）。新增 2 人對戰遊戲＝這三檔 + page + i18n namespace + `libs/site-nav.ts` GAMES 一行（header 下拉與 `/games` index 頁共用，另補 `GamesHub.items` 描述）+ `GameId` union。信封 `{ game, type, data }`，`game` 必填、上行帶下行過濾。server 權威裁判，前端不複刻規則（含合法步提示），收 `move_made` 才更新盤面。**協定看 monorepo 根 `protocol/games-wire.md`（唯一準，進版控，含各遊戲差異）；象棋棋規以後端引擎為準**
- `app/[locale]/(public)/games/avalon/` — **阿瓦隆**（5–10 人社交推理），**不走 `_shared` 2 人框架**（無大廳/桌位/Fischer），但底層機械邏輯共用 `_shared/useRoomBase`。自成一套：`useAvalonRoom`（N 人房狀態機 + 私有角色 + 階段機 team_building/team_vote/quest/assassinate）、`AvalonLobby`/`AvalonRoom`/`AvalonPlay`/`AvalonChat`。⚠️**私有角色（`role_assigned`）只存本地、絕不外送/不渲染給他人**；切頁送 `leave_room`。協定看 `protocol/games-wire.md`「三、阿瓦隆」
- `app/[locale]/(public)/games/farm/` — **農場經營**（2–4 人 worker-placement，完全資訊）。同 avalon 的 N 人房模型（`useFarmRoom`/`FarmLobby`/`FarmRoom`/`FarmPlay`）；**每動作後 server 廣播完整 `state`，前端整盤重繪**（無私有狀態、無 delta）。盤面抽象（phase-1 無逐格座標，用數量+圖示）。複合動作 `sow`/`build_rooms`/`fences` 有 input 表單。`state`/`room_update` 逐人帶 `your_seat`（判輪到我 `current_player===your_seat`、推 host `your_seat===host_seat`）。協定看 `protocol/games-wire.md`「四、農場經營」

---

## 已知的技術債（2026-07-31 全前端盤點，刻意未做）

盤過 82 個 components / 71 個 page / 23 個 api 檔的結果（**2026-09-11 重數：components 51 檔、page 56、api 21** —— 該日移除記帳／發票／樂透三個功能，對不上不代表這些項目消失，各條下方的計數才是現值）。下列都**已確認存在、有具體落點**，不必再重新調查：

### 跨模組可抽共用
- **`libs/fetchApi.ts` 與 `libs/createAuthRequest.ts` 的核心該合併**。剩下三個關鍵分歧：timeout（10s vs 30s）、空 body（`res.json()` 直接 throw vs `text()` 後回 null）、401/403 處理。**錯誤物件那一項已在 2026-08-07 收斂** —— 兩邊都附 `status` / `errorData`（見上方「錯誤形狀」），`contact/actions.ts` 也已改用 `apiErrorStatus(e)`，不再字串比對 `429`。`adminRequest` / `memberRequest` 本身已是 `createAuthRequest` 的薄組態，重複早就消除了。
- **`err as Error & { status?: number; errorData?: {...} }` 這種 inline cast 還剩 7 處**（`admin/login/page.tsx` ×4、`change-password` / `passkeys` / `api/torrents.ts` 各 1，2026-08-26 重數，2026-09-25 複驗相同）。**型別與 helper 已經有了** —— `libs/api-error.ts` 匯出 `ApiError` / `apiErrorStatus` / `apiErrorMessage`，這 7 處只是還沒改過去。
- ~~**沒有 `cn()`，`inputClass` 在 15 個檔案各自宣告**~~（2026-08-30 收斂：`libs/cn.ts` + `libs/input-styles.ts` 的 `ADMIN_INPUT` / `ADMIN_FILTER_INPUT` / `PUBLIC_INPUT` 吃掉 14 份，用法見「樣式慣例」。剩下的 `contact/contact-form.tsx` 是列明例外 —— 表單不在卡片上，底色跟頁面漸層走）。
- **缺 `libs/format-number.ts` 與公開端的日期 formatter**：金額格式化散 5 處、`Intl.DateTimeFormat(locale, {... Asia/Taipei})` 逐字重複 3 份、「今天（台北）」2 份。（~~bytes 格式化 3 份~~ 已收斂進 `libs/format-bytes.ts`，torrents 的 3 個消費點都 import 它。）admin 側已有 `libs/admin-datetime.ts` 當範本（但 metrics 那 4 處仍繞過它自建 formatter）。
- ~~`{ data: T[]; total: number }` 重複定義~~（2026-08-03：`types/pagination.ts` 的 `PaginatedResponse<T>` 收斂了全部 7 份 inline 定義；resource 專屬名稱如 `TorrentPaginatedResponse` 保留為 type alias，消費端不必改）。另有 4 個後端契約型別（`PublicSettings` / `RosterResponse` / `TorrentActionResult` / `TorrentLinksResult`）住在 `api/` 而非 `types/`。
- **avalon 與 farm 之間約 150–170 行可抽「N 人房」層**：`AvalonLobby`(112) ↔ `FarmLobby`(89) 的 diff 有 4 個 hunk 純粹是排版差異，實質差異只有 avalon 多兩個 checkbox；玩家列表區塊（`grid grid-cols-2` + `Crown` host 標記 + className）逐字相同。另 avalon/farm 的 `avalon-types.ts` / `farm-types.ts` 各自定義了 7 個逐字相同的 room 型別（`RoomSummary` / `PlayerInfo` / `RoomListData` …），`_shared/wire.ts` 已存在卻沒收進去（`ErrorData` 甚至是第 3 份）。
- **`api/stocks.ts` 不存在**：stocks 的 4 個純讀取函式住在 `app/admin/(main)/stocks/actions.ts`；`settings` 的 admin 讀取也在 `settings/actions.ts`。這是「一資源一檔」的最後破口。
- **`api/settings.ts` 用 `next: { revalidate: 60 }` 但不標 tag**，卻在註解宣稱靠 `updateSiteTheme` 的 `revalidatePath('/', 'layout')` 立即失效 —— 這條依賴很脆弱，改 `tags:['settings']` + `updateTag('settings')` 才與 blogs 一致。另 `stocks/actions.ts` 的 `revalidatePath("/")` 範圍過大（刷整站首頁去更新後台清單）。

### 拆檔
- ~~三支大檔~~（2026-09-25 拆完，檔案都 colocate 在原 route 目錄；狀態全留在原本的主元件，子元件只收 props，JSX 逐字搬移）：
  - `vocab/vocab-client.tsx`（1149 → 493 行）：`model.ts`（型別 + `hasLives` / `hasTimer`）、`ui.tsx`（共用小元件 + `T` 型別）、`play-cards.tsx`（作答中）、`result-cards.tsx`（結算）、`leaderboard-card.tsx`、`mistake-book.tsx`
  - `tools/roster/page.tsx`（576 → 237）：`settings.ts`（localStorage 設定讀寫 + `clampNumber` / `digitsOnly`，純函式）、`roster-form.tsx`（參數表單）、`roster-result.tsx`（結果區）；子元件各自 `useTranslations("Roster")`，不經 props 傳 `t`
  - `admin/(main)/logs/logs-client.tsx`（515 → 275）：`log-groups.ts`（`groupConsecutive` / `sortedFields`，純函式）、`log-row.tsx`（一列 + 展開面板）、`trace-drawer.tsx`（軌跡抽屜；`useDialog` 仍在主元件，ref 以 prop 傳入，焦點還原行為不變）
- 拆完後除 `vocab-client.tsx` 本身外，最大的是 `admin/(main)/vocab/vocab-admin-client.tsx`（356）與 `components/header.tsx`（342），都還在單一關注點內，未動。

### UI 一致性
- ~~**admin 表格外框分裂**~~（2026-08-17：手刻 wrapper 已歸零）、~~**卡片底色打架**~~（2026-08-30：`bg-white dark:bg-neutral-800` 的 29 處卡片全改成 `-900`，對齊 `AdminTableContainer`；admin-only 但住在 `components/blogs` / `components/images` 的 3 支 modal 也隨 `<Modal surface="admin">` 一起改過來）。**後台卡片底色現在只有一個值：`bg-white dark:bg-neutral-900`。** 沒被改的 `dark:bg-neutral-800` 是別的角色，別順手一起改：表頭 `bg-neutral-100 dark:bg-neutral-800`（11 處，要比卡片淺一階才看得出表頭）、篩選列底 `bg-neutral-50 dark:bg-neutral-800/50`（7 處）、停用狀態的 chip、sidebar 的半透明底。
- ~~**15 處手寫 `fixed inset-0` modal 殼**，`useDialog` 只有 8 個檔案在用~~（2026-08-30：`components/modal.tsx` 收掉 9 個置中彈窗，**其中 4 個原本完全沒有 focus trap／背景捲動鎖／Esc**（`HowToPlay`、`stock-history-table`、vocab 編輯框等）。用法與「為什麼不收抽屜類」見「樣式慣例」）。`fixed inset-0` 現在剩 7 個檔案 / 8 處：`components/modal.tsx` 自己一處，其餘是刻意不收的非置中浮層（抽屜／header 手機選單／命令面板／遊戲結局遮罩）。
- **admin 11 個 `loading.tsx` 沒用 `components/loading/*`**（2026-09-05 重數；2026-09-04 移除收盤價查詢頁後由 12 降為 11），其中 4 個 stocks 的內容幾乎相同、3 個（`games` / `ws` / `members/[id]`）手抄了 `table-skeleton.tsx` 的 `<table className="w-full border-collapse …">`（＝**後台**僅剩的手寫 `<table>`；前台另有 3 支 `portfolio/stock-history-table.tsx`、`tools/roster/roster-table.tsx`、`roster-stats.tsx`，那是**列明例外** —— `components/admin/table.tsx` 是後台專用的視覺規格，前台套上去反而不一致）。
- **`games/page.tsx` 與 `tools/page.tsx` 完全同構**（diff 只有 8 行：`GAMES`↔`TOOLS`、namespace、路徑），可抽 `<NavHubPage>`。
- `useFilterUrl` 只有 5 個消費點（logs / audit_logs / gov_tenders / vocab / blogs），但 `messages-client` / `blog-comments-client` / `metrics-audit-panel` 也有 filter 卻沒同步 URL（覆蓋不足，非重複）。
- `admin/` 28 個 page **完全沒有 error boundary**（2026-09-05 重數；前台只有 2 個 `error.tsx`）。
- **`(public)/loading.tsx` 是裸的 `Loader2` spinner，不是骨架**（2026-08-30）。它是整個前台的 fallback ——沒有自帶 `loading.tsx` 的頁全吃它，等於違反「前台資料頁一律用 `PublicPageSkeleton`」那條自家規則。`blogs/` 那 3 支自訂骨架則是**列明例外**（檔頭有註解：要對齊 blog 卡片／文章版面，`list`/`cards`/`form` 三種 variant 都套不上），別把它們一起改掉。
- `vocab` 與 `vocab-ja` 的 page **重複已收斂**（2026-08-21）：資料抓取集中在 `vocab/load.ts` 的 `loadVocabPage(language)`，兩支 page 各剩約 20 行（只差 `language` 與 JMdict 標註）。仍有的是 `vocab-ja` 跨目錄 import `../vocab/{vocab-client,load}` —— 要徹底解掉得做成 `vocab/[lang]/page.tsx`，但兩頁路徑已對外（含 sitemap / site-nav），改動範圍遠大於剩下的重複量。

### 已查證「不是問題」，別再動
- **avalon 的 `iAmHost` 用本地 `useState` 是正確的**，不是偷懶。後端 `games/common/room.rs` 依 `K::SEAT_IN_ROOM_UPDATE` 決定要不要逐人注入 `your_seat`，farm 設 `true`、avalon 吃預設 `false`；avalon 的 seat 從私有 `role_assigned` 拿，開局前只需要 host 旗標。而且 `room.rs` 的 `leave_room` 是 **host 離開＝解散房間**（`host_left`），所以「重整後回到房內但看不到開始按鈕」的情境不存在 —— 重整後根本沒有房可以回。farm 需要 `your_seat` 是因為它要判 `myTurn`（`current_player === mySeat`）且沒有私有訊息可夾帶。**兩邊的差異是設計，不是疏漏。**
- i18n 三語系 key 完全同步（**822 / 822 / 822，38 個 namespace**，2026-09-13 首頁補「最新文章」後重數；2026-09-11 移除記帳／發票／樂透後是 819 / 38；2026-09-09 是 1050 / 42、2026-08-30 是 1045 / 42、2026-08-26 是 1027 / 42、2026-08-17 是 883 / 40、2026-07-31 是 865 / 39），namespace 與 code 中的用量雙向吻合，沒有孤兒 namespace。**靜態掃描會誤報約 152 個「沒人用」的 key，那些是 30 處動態組 key（`t(\`items.${key}\`)` 之類）**，刪之前務必逐一確認。
- `types/` 的 barrel 是 100% 遵守的（`from '@/types/xxx'` 零命中）。
- `api/` 21 檔全部有 `"use server"`、全部 named exports；手寫 `fetch(` 只有 `api/github.ts` 一支，那是刻意的（打 GitHub 公開 API，套不上 `fetchApi`，理由見上方「API 請求分層」）。
- `usePagedList` 的 9 個消費點用法一致（2026-09-11 重數，移除記帳／發票／樂透前是 12），沒有頁面自己手刻「載入更多」。
- `next/image` 慣例乾淨：5 處原生 `<img>` 全部帶 eslint-disable 且全部是列明例外（blob 預覽 / 外部 avatar）。
- `gray` / `slate` / `zinc` / `indigo` 全庫 0 命中。
- **`useAlarm` 與 `useTimer` 的倒數狀態機早就抽好了** —— `hooks/useCountdownCore.ts`（對齊秒邊界的 tick、剩餘秒數、到點轉響鈴、暫停／清除），兩支都用它。舊版技術債清單曾把它列成「可抽 `useCountdownTo`」，那條在 2026-08-30 確認過期並刪除，別再提案。
- **`components/loading/*` 的 4 支骨架都有多個消費點**，不是為抽而抽（`PublicPageSkeleton` 前台全用、`table-skeleton` 的兩支後台 11 檔、`form-skeleton` 4 檔、`chart-page-skeleton` 2 檔）。
- **全庫沒有孤兒模組**：`components` / `libs` / `hooks` / `api` / `types` 逐檔掃 alias + 相對路徑 import，零死碼（2026-08-30）。
