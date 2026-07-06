// Auth（GET /admin/auth/me）
export interface AuthUser {
  id: number;
  name: string;
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

// User（後台管理員；name 為登入識別，email 選填）
export interface User {
  id: string;
  name: string;
  email?: string | null;
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
