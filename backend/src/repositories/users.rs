use crate::{
    errors::AppError,
    structs::{roles::Role, users::{NewUser, User}},
};
use sqlx::{Pool, Postgres};

pub async fn get_users(pool: &Pool<Postgres>) -> Result<Vec<User>, AppError> {
    Ok(sqlx::query_as("SELECT id, name, email FROM users")
        .fetch_all(pool)
        .await?)
}

/// 認證 middleware 用：以 id 一次查出 (name, 是否 super_admin)。
/// 找不到（帳號已刪但 token/session 未過期）回 None。
pub async fn get_identity_by_id(
    pool: &Pool<Postgres>,
    id: i64,
) -> Result<Option<(String, bool)>, AppError> {
    Ok(sqlx::query_as(
        r#"
        SELECT u.name,
               EXISTS (
                   SELECT 1 FROM user_roles ur
                   JOIN roles r ON ur.role_id = r.id
                   WHERE ur.user_id = u.id AND r.name = 'super_admin'
               ) AS is_super_admin
        FROM users u
        WHERE u.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

/// 登入用：以 name（唯一）查出 (id, password hash)；帳號不存在回 None。
pub async fn get_credentials_by_name(
    pool: &Pool<Postgres>,
    name: &str,
) -> Result<Option<(i64, String)>, AppError> {
    Ok(sqlx::query_as("SELECT id, password FROM users WHERE name = $1 LIMIT 1")
        .bind(name)
        .fetch_optional(pool)
        .await?)
}

/// passkey 註冊 begin 用：以 id 查出 (webauthn user handle, name)。
pub async fn get_webauthn_identity_by_id(
    pool: &Pool<Postgres>,
    id: i64,
) -> Result<Option<(uuid::Uuid, String)>, AppError> {
    Ok(sqlx::query_as("SELECT webauthn_user_handle, name FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

/// passkey 登入 finish 用：以 user handle 反查 user id。
pub async fn get_id_by_webauthn_handle(
    pool: &Pool<Postgres>,
    handle: uuid::Uuid,
) -> Result<Option<i64>, AppError> {
    Ok(sqlx::query_scalar("SELECT id FROM users WHERE webauthn_user_handle = $1")
        .bind(handle)
        .fetch_optional(pool)
        .await?)
}

/// 改密碼用：以 id 取現有 password hash。
pub async fn get_password_by_id(pool: &Pool<Postgres>, id: i64) -> Result<Option<String>, AppError> {
    Ok(sqlx::query_scalar("SELECT password FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
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

pub async fn delete_user(pool: &Pool<Postgres>, user_id: i64) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
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

pub async fn update_password(
    pool: &Pool<Postgres>,
    id: i64,
    new_hash: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password = $1 WHERE id = $2")
        .bind(new_hash)
        .bind(id)
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
