use crate::{errors::{AppError, AuthError}, structs::roles::Perm};
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

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub email: String,
    pub permissions: Vec<String>,
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
}
