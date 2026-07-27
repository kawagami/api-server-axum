// WS Notification
export type WsEventType = 'stock_completed' | 'stock_failed' | 'blog_created' | 'user_joined' | 'user_left' | 'admin_message';

export interface WsNotification {
  id: number;
  type: WsEventType;
  data: unknown;
}

// user_joined / user_left 的 payload（後端 routes/ws.rs broadcast_to_admins，只推給已登入連線）。
// user_joined 帶滿 WsConnection 的所有欄位，admin 頁可直接據此插入新列；user_left 只帶識別欄位。
export interface WsUserEventData {
  addr: string;
  user_email: string | null;
  real_ip: string;
  connected_at?: string;
  user_agent?: string;
}

// Raw WS frame from stock notification server
export interface WsMessage {
  type: string;
  data: unknown;
}

// WS online connection（後端 DisplayTrackedConnection，已依 connected_at 新→舊排序）
export interface WsConnection {
  addr: string;
  user_email: string | null;
  // CF-Connecting-IP 優先，沒有才退回 socket 來源 IP
  real_ip: string;
  // ISO-8601 毫秒 UTC 字串
  connected_at: string;
  user_agent: string;
}
