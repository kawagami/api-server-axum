// Image (server storage)
export interface Image {
  id: string;
  storage_key: string;
  url: string;
  status?: string;
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

/** 操作者身分。member 的稽核 2026-08-09 起才記，在那之前的列一律 admin */
export type AuditActorType = 'admin' | 'member';

export interface AuditLog {
  id: number;
  actor_type: AuditActorType;
  /** admin 是顯示名；member 是 `member#{id}`（後端不為了稽核多打一次 DB 取名字） */
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
  // 對應 `x-request-id` / 錯誤 body 的 request_id；非請求路徑的 log（排程 job、啟動期）為 null
  request_id: string | null;
  // event 與 span 的其餘 field。`self` = 真正的錯誤細節（message 只是固定字串）、
  // 另有 method / path / latency 等，內容依 log 來源而異，故不收斂成具名型別
  fields: Record<string, unknown> | null;
  created_at: string;
}
