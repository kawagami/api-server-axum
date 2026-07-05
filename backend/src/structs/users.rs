use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Serialize, FromRow)]
pub struct DbUser {
    pub id: i64,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
    /// 建立時要指派的角色 id；省略或空陣列時走 app_settings `new_user_default_roles`
    #[serde(default)]
    pub role_ids: Vec<i32>,
}
