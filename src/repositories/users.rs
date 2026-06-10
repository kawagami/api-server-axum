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
    default_role_id: i32,
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

    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(default_role_id)
        .execute(&mut *tx)
        .await?;

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
