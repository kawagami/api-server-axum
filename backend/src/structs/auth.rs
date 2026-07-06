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

#[derive(Clone)]
pub struct CurrentUser {
    pub email: String,
    pub password_hash: String,
}

#[derive(Deserialize)]
pub struct SignInData {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordData {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: i64,
    pub email: String,
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
