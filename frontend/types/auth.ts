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

// User
export interface User {
  id: string;
  email: string;
  name?: string;
  created_at?: string;
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
