use crate::errors::{AppError, AuthError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub email: String,
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
    pub fn require_permission(&self, permission: &str) -> Result<(), AppError> {
        if self.permissions.iter().any(|p| p == permission) {
            Ok(())
        } else {
            Err(AppError::AuthError(AuthError::Forbidden))
        }
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }
}
