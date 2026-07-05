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
