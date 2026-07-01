// Portfolio
export interface HistoryRecord {
  date: string;
  close: number;
  adjusted_cost: number;
  pnl: number;
  pnl_pct: number;
}

export interface PortfolioEntry {
  id: string;
  member_id: number;
  stock_code: string;
  buy_date: string;
  cost_per_share: number;
  shares: number;
  created_at: string;
  updated_at: string;
}

export interface PortfolioEntryInput {
  stock_code: string;
  buy_date: string;
  cost_per_share: number;
  shares: number;
}

export interface PortfolioSummaryEntry extends PortfolioEntry {
  stock_name: string;
  current_price: number | null;
  current_value: number | null;
  pnl: number | null;
  pnl_pct: number | null;
}

// Ledger（記帳）
export type LedgerKind = 'income' | 'expense';

export type LedgerSource = 'manual' | 'invoice_qr';

export interface LedgerEntry {
  id: string;
  member_id: number;
  kind: LedgerKind;
  amount: string; // 十進位字串，避免浮點誤差
  category: string;
  note: string | null;
  occurred_at: string; // YYYY-MM-DD
  // 掃發票匯入會帶這三欄；手動記帳 invoice_number/seller_tax_id 為 null、source='manual'
  invoice_number?: string | null;
  seller_tax_id?: string | null;
  source?: LedgerSource;
  created_at: string;
  updated_at: string;
}

export interface LedgerInput {
  kind: LedgerKind;
  amount: string;
  category: string;
  note?: string | null;
  occurred_at: string;
}

export interface LedgerCategoryOption {
  value: string;
  label: string;
}

export interface LedgerCategories {
  income: LedgerCategoryOption[];
  expense: LedgerCategoryOption[];
}

export interface LedgerCategoryTotal {
  kind: LedgerKind;
  category: string;
  total: string;
}

export interface LedgerMonthly {
  month: string; // YYYY-MM
  income: string;
  expense: string;
}

export interface LedgerSummary {
  total_income: string;
  total_expense: string;
  balance: string;
  by_category: LedgerCategoryTotal[];
  monthly: LedgerMonthly[];
}

// 統一發票登錄 + 對獎（解耦於記帳，走 POST /member/invoices）
export type InvoiceSource = 'qr' | 'barcode' | 'manual';

// 中獎獎別：special 特別獎、grand 特獎、first 頭獎、second~sixth 二~六獎、additional_sixth 增開六獎
export type PrizeTier =
  | 'special'
  | 'grand'
  | 'first'
  | 'second'
  | 'third'
  | 'fourth'
  | 'fifth'
  | 'sixth'
  | 'additional_sixth';

export interface Invoice {
  id: string;
  member_id: number;
  invoice_number: string;
  invoice_date: string; // 西元 YYYY-MM-DD
  period: string; // 對獎期別 key（期末偶數月 YYYYMM），後端算好
  amount: string | null; // 十進位字串，可為 null
  seller_tax_id: string | null;
  source: InvoiceSource;
  ledger_entry_id: string | null; // record_as_expense 時連結的記帳 id
  lottery_checked: boolean; // 該期是否已開獎並對過
  prize_tier: PrizeTier | null; // checked=true 且為 null = 確定未中
  notified_at: string | null;
  created_at: string;
  updated_at: string;
}

// 登錄發票（POST /member/invoices）
export interface InvoiceInput {
  invoice_number: string;
  invoice_date: string;
  amount?: string | null;
  seller_tax_id?: string | null;
  source: InvoiceSource;
  record_as_expense?: boolean; // true 時同時建一筆 ledger 支出並連結
  category?: string; // record_as_expense=true 時用，省略 → other
  note?: string | null;
}

export interface InvoiceListParams {
  period?: string; // YYYYMM
  won?: boolean; // true=只看中獎、false=只看未中、省略=全部
  page?: number;
  per_page?: number;
}

// 大樂透 / 威力彩 選號登錄 + 自動對獎（POST /member/lotto；與發票對獎為不同功能）
export type LottoGame = 'lotto649' | 'super_lotto638';
export type LottoSource = 'qr' | 'manual';
export type LottoStatus = 'pending' | 'won' | 'lost';

// 中獎獎別 key；須搭配 game 解讀（lotto649 到 seventh+general、super_lotto638 到 ninth）
export type LottoPrizeTier =
  | 'first'
  | 'second'
  | 'third'
  | 'fourth'
  | 'fifth'
  | 'sixth'
  | 'seventh'
  | 'eighth'
  | 'ninth'
  | 'general';

// 一注：一般號/第一區 6 個相異；威力彩第二區 second(1-8)，大樂透 second 為 null
export interface LottoNote {
  picks: number[];
  second: number | null;
}

export interface LottoTicket {
  id: string;
  member_id: number;
  game: LottoGame;
  draw_date: string; // 這注要對的開獎日 YYYY-MM-DD
  picks: number[];
  second: number | null;
  source: LottoSource;
  checked: boolean; // 該期是否已開獎並對過
  prize_tier: LottoPrizeTier | null; // checked=true 且為 null = 確定未中
  notified_at: string | null;
  created_at: string;
  updated_at: string;
}

// 批次登錄（POST /member/lotto）：整批共用 game/draw_date/source，notes 帶多注
export interface LottoInput {
  game: LottoGame;
  draw_date: string;
  source: LottoSource;
  notes: LottoNote[];
}

export interface LottoListParams {
  game?: LottoGame;
  status?: LottoStatus; // pending=未開獎、won=中獎、lost=未中、省略=全部
  page?: number;
  per_page?: number;
}

export interface LottoDraw {
  game: LottoGame;
  period: string; // 台彩期別字串，資訊用
  draw_date: string;
  main_nums: number[]; // 一般號/第一區（已排序）
  special: number; // 大樂透=特別號；威力彩=第二區號
}

export interface LottoDrawParams {
  game?: LottoGame;
  limit?: number;
}

// Blog
export interface Toc {
  id: string;
  level: number;
  text: string;
}

export interface Blog {
  id: string;
  markdown: string;
  tags: string[];
  tocs: string[];
  created_at?: string;
  updated_at?: string;
}

export interface BlogPaginatedResponse {
  total: number;
  page: number;
  per_page: number;
  data: Blog[];
}

export interface BlogInput {
  markdown: string;
  tags: string[];
  tocs: Toc[];
}

// Image (server storage)
export interface Image {
  id: string;
  storage_key: string;
  url: string;
  status?: string;
}

// User
export interface User {
  id: string;
  email: string;
  name?: string;
  created_at?: string;
}

// HackMD — fields use snake_case matching API response
export interface HackmdNote {
  id: string;
  title: string;
  tags: string[];
  publish_link: string;
  last_changed_at: string;
}

export interface HackmdTag {
  id: string;
  name: string;
}

// Stock
export interface StockDayAll {
  id: number;
  trade_date: string;
  stock_code: string;
  stock_name: string;
  trade_volume: number | null;
  trade_amount: number | null;
  open_price: number | null;
  high_price: number | null;
  low_price: number | null;
  close_price: number | null;
  price_change: number | null;
  transaction_count: number | null;
}

export interface StockBuybackPeriod {
  stock_no: string;
  start_date: string;
  end_date: string;
}

export interface StockChange {
  id: string | number;
  stock_no: string;
  stock_name: string;
  status: string;
  start_date: string;
  start_price: number;
  end_date: string;
  end_price: number;
  change: number;
}

export interface StockChangePaginatedResponse {
  data: StockChange[];
  total: number;
}

// Auth
export interface AuthUser {
  email: string;
  permissions: string[];
}

export interface Permission {
  id: number;
  resource: string;
  action: string;
  description?: string;
}

export interface Role {
  id: number;
  name: string;
  description?: string;
  permissions?: Permission[];
}

// Member
export interface Member {
  id: number;
  name: string;
  email: string | null;
  avatar_url: string | null;
  created_at: string;
}

export interface MemberDetail {
  id: number;
  name: string;
  email: string | null;
  avatar_url: string | null;
  created_at: string;
  providers: string[];
  lottery_notify_enabled: boolean; // 發票中獎 email 通知開關（預設關閉）
  lotto_notify_enabled: boolean; // 樂透選號中獎 email 通知開關（預設關閉）
}

// WS Notification
export type WsEventType = 'stock_completed' | 'stock_failed' | 'blog_created' | 'user_joined' | 'user_left' | 'admin_message';

export interface WsNotification {
  id: number;
  type: WsEventType;
  data: unknown;
}

export interface WsUserEventData {
  addr: string;
  user_email: string | null;
}

// Raw WS frame from stock notification server
export interface WsMessage {
  type: string;
  data: unknown;
}

// WS online connection
export interface WsConnection {
  addr: string;
  user_email: string | null;
}

// Torrent
export type TorrentStatus = 'pending' | 'downloading' | 'completed' | 'failed';

export interface TorrentFile {
  index: number;
  path: string;
  size: number;
}

// 即時進度（不存 DB，僅進行中任務的詳情 API 與 WS 推播帶）
export interface TorrentLive {
  progress: number;
  progress_bytes: number;
  total_bytes: number;
  down_speed: string;
  peers: number;
}

export interface Torrent {
  id: number;
  info_hash: string;
  magnet_uri: string;
  name: string | null;
  status: TorrentStatus;
  total_size: number | null;
  files: TorrentFile[] | null;
  error: string | null;
  created_by: string;
  created_at: string;
  completed_at: string | null;
  live?: TorrentLive;
}

export interface TorrentPaginatedResponse {
  data: Torrent[];
  total: number;
}

export interface TorrentDownloadLink {
  file_index: number;
  path: string;
  size: number;
  url: string;
  expires_at: string;
}

export interface TorrentStorage {
  disk: { total_bytes: number; available_bytes: number };
  torrent: { used_bytes: number; max_bytes: number };
}

export interface TorrentProgressEvent extends TorrentLive {
  id: number;
  name: string;
}

export interface TorrentCompletedEvent {
  id: number;
  name: string;
  total_size: number;
}

export interface TorrentFailedEvent {
  id: number;
  name: string;
  reason: string;
}

// Setting
export interface Setting {
  key: string;
  value: string;
  description: string;
  category: string;
}

export type SettingsResponse = Record<string, Setting[]>;

// Audit Log
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

export interface AuditLog {
  id: number;
  user_email: string;
  method: string;
  path: string;
  query: string | null;
  status_code: number;
  created_at: string;
}

// Games overview（admin 即時對局總覽）
export interface GameOverview {
  game: string;
  waiting: number;
  playing: number;
  players_in_game: number;
  queued: number;
  lobby: number;
}

// 到訪統計（admin）：HLL 不重複到訪，今日來自 Redis、昨日以前來自 DB，後端已合好
export interface VisitorDayCount {
  date: string;
  unique_visitors: number;
}

export interface VisitorStats {
  today: VisitorDayCount;
  last_n_days_unique: number; // 跨日去重總數，≤ history 每日相加
  history: VisitorDayCount[];
}

// Log
export type LogLevel = 'INFO' | 'WARN' | 'ERROR';

export interface Log {
  id: number;
  level: LogLevel;
  message: string;
  target: string;
  file: string;
  line: number;
  created_at: string;
}
