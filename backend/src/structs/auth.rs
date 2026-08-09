use crate::{errors::{AppError, AuthError, RequestError}, structs::roles::Perm};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshClaims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
    pub jti: String,
}

#[derive(Deserialize)]
pub struct SignInData {
    pub name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordData {
    pub current_password: String,
    pub new_password: String,
}

/// 認證用身分快取的內容（Redis `user:identity:{id}`）。
///
/// 原本只快取 `permissions`，`name` 與 `is_super_admin` 每個請求都打一次 PG
/// （`users_repo::get_identity_by_id`）—— 於是「權限快取命中」也還是 1 次 DB round-trip，
/// 而 `/admin/*` 的每個請求都會走這條。三個值的失效條件完全相同（`set_user_roles` /
/// `set_role_permissions` / `delete_user` / `delete_role`），沒有理由分兩份存。
///
/// ⚠️ **`name` 能被快取的前提是 `users.name` 不可變**（全 repo 只有 password 有 UPDATE）。
/// 哪天加了改名端點，那支必須呼叫 `redis::invalidate_user_identity`。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedIdentity {
    pub name: String,
    pub is_super_admin: bool,
    pub permissions: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: i64,
    /// 登入識別 + 顯示用（稽核 / WS / torrent created_by）；不再用 email
    pub name: String,
    pub permissions: Vec<String>,
    /// super_admin 角色 → 資料隔離下看得到/管得到所有 admin 的資料
    pub is_super_admin: bool,
}

impl AuthenticatedUser {
    pub fn has_permission(&self, perm: Perm) -> bool {
        self.permissions.iter().any(|p| p == perm.as_str())
    }

    pub fn require_permission(&self, perm: Perm) -> Result<(), AppError> {
        if self.has_permission(perm) {
            Ok(())
        } else {
            Err(AppError::AuthError(AuthError::Forbidden))
        }
    }

    /// 資料隔離用的擁有者過濾值：super_admin 回 None（看全部），否則回 Some(自己的 id)。
    pub fn owner_filter(&self) -> Option<i64> {
        if self.is_super_admin { None } else { Some(self.id) }
    }

    /// 是否可存取某筆資料：super_admin 全可；否則擁有者須為自己。
    pub fn can_access(&self, owner_id: Option<i64>) -> bool {
        self.is_super_admin || owner_id == Some(self.id)
    }

    /// 非擁有者（且非 super_admin）→ 回 NotFound（不洩漏他人資源存在）。
    pub fn require_owner(&self, owner_id: Option<i64>) -> Result<(), AppError> {
        if self.can_access(owner_id) {
            Ok(())
        } else {
            Err(AppError::RequestError(RequestError::NotFound))
        }
    }
}
