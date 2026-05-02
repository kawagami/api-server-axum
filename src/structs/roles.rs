use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub enum Perm {
    RoleRead,
    RoleAssign,
    RoleCreate,
    RoleUpdate,
    RoleDelete,
}

impl Perm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RoleRead   => "role:read",
            Self::RoleAssign => "role:assign",
            Self::RoleCreate => "role:create",
            Self::RoleUpdate => "role:update",
            Self::RoleDelete => "role:delete",
        }
    }
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
pub struct Permission {
    pub id: i32,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct RoleWithPermissions {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Deserialize)]
pub struct NewRole {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct SetRolePermissions {
    pub permission_ids: Vec<i32>,
}

#[derive(Deserialize)]
pub struct SetUserRoles {
    pub role_ids: Vec<i32>,
}
