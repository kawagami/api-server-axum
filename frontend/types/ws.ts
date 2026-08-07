// 後端會推的所有事件名，與 backend/src/structs/ws.rs 的 `WsEvent` enum 一一對應
// （新增事件兩邊要同步加）。這裡原本只列了 6 個：漏掉 4 個 torrent 事件、
// 卻多了一個當時後端手寫字串沒進 enum 的 admin_message，兩邊各自漂移。
export type WsEventType =
  | 'stock_completed'
  | 'stock_failed'
  | 'blog_created'
  | 'user_joined'
  | 'user_left'
  | 'torrent_progress'
  | 'torrent_completed'
  | 'torrent_failed'
  | 'torrent_retrying'
  | 'admin_message';

// 會彈全站 toast / 進通知列表的子集。torrent 進度只有後台 torrents 頁在看，
// 每秒推一次，彈成 toast 會洗版
export type WsNotifyEventType = Exclude<WsEventType, `torrent_${string}`>;

export interface WsNotification {
  id: number;
  type: WsNotifyEventType;
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
