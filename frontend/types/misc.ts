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
