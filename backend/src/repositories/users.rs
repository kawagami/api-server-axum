use crate::{
    errors::AppError,
    structs::{roles::Role, users::{DbUser, NewUser, User}},
};
use sqlx::{Pool, Postgres};

pub async fn get_users(pool: &Pool<Postgres>) -> Result<Vec<User>, AppError> {
    Ok(sqlx::query_as("SELECT id, name, email FROM users")
        .fetch_all(pool)
        .await?)
}

/// 認證 middleware 用：以 email 一次查出 (user id, 是否 super_admin)。
/// email 有唯一索引；找不到（帳號已刪但 token/session 未過期）回 None。
pub async fn get_identity_by_email(
    pool: &Pool<Postgres>,
    email: &str,
) -> Result<Option<(i64, bool)>, AppError> {
    Ok(sqlx::query_as(
        r#"
        SELECT u.id,
               EXISTS (
                   SELECT 1 FROM user_roles ur
                   JOIN roles r ON ur.role_id = r.id
                   WHERE ur.user_id = u.id AND r.name = 'super_admin'
               ) AS is_super_admin
        FROM users u
        WHERE u.email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?)
}

pub async fn check_email_exists(pool: &Pool<Postgres>, email: &str) -> Result<DbUser, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, email, password FROM users WHERE email = $1 LIMIT 1",
    )
    .bind(email)
    .fetch_one(pool)
    .await?)
}

pub async fn create_user(
    pool: &Pool<Postgres>,
    new_user: NewUser,
    role_ids: &[i32],
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let (user_id,): (i64,) = sqlx::query_as(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&new_user.name)
    .bind(&new_user.email)
    .bind(&new_user.password)
    .fetch_one(&mut *tx)
    .await?;

    if !role_ids.is_empty() {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id)
             SELECT $1, unnest($2::int[])
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(role_ids)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn delete_user(pool: &Pool<Postgres>, user_id: i64) -> Result<String, AppError> {
    let mut tx = pool.begin().await?;

    let (email,): (String,) = sqlx::query_as("SELECT email FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(email)
}

pub async fn get_user_roles(pool: &Pool<Postgres>, user_id: i64) -> Result<Vec<Role>, AppError> {
    Ok(sqlx::query_as(
        "SELECT r.id, r.name, r.description
         FROM roles r
         JOIN user_roles ur ON ur.role_id = r.id
         WHERE ur.user_id = $1
         ORDER BY r.id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_email_by_id(pool: &Pool<Postgres>, user_id: i64) -> Result<String, AppError> {
    let (email,): (String,) =
        sqlx::query_as("SELECT email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(email)
}

pub async fn update_password(
    pool: &Pool<Postgres>,
    email: &str,
    new_hash: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password = $1 WHERE email = $2")
        .bind(new_hash)
        .bind(email)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_user_roles(
    pool: &Pool<Postgres>,
    user_id: i64,
    role_ids: &[i32],
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

    if count == 0 {
        return Err(AppError::RequestError(crate::errors::RequestError::NotFound));
    }

    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    if !role_ids.is_empty() {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id)
             SELECT $1, unnest($2::int[])
             ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(role_ids)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
