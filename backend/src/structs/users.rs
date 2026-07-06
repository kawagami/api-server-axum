use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct NewUser {
    pub name: String,
    /// 選填聯絡信箱（登入識別改用 name，email 不再必填）
    #[serde(default)]
    pub email: Option<String>,
    pub password: String,
    /// 建立時要指派的角色 id；省略或空陣列時走 app_settings `new_user_default_roles`
    #[serde(default)]
    pub role_ids: Vec<i32>,
}
