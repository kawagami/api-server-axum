use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
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
