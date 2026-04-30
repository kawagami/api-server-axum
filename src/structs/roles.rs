use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
