use crate::{
    errors::AppError,
    state::AppStateV2,
    structs::users::{DbUser, NewUser, User},
};

pub async fn get_users(state: &AppStateV2) -> Result<Vec<User>, AppError> {
    Ok(sqlx::query_as("SELECT id, name, email FROM users")
        .fetch_all(state.get_pool())
        .await?)
}

pub async fn check_email_exists(state: &AppStateV2, email: &str) -> Result<DbUser, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, email, password FROM users WHERE email = $1 LIMIT 1",
    )
    .bind(email)
    .fetch_one(state.get_pool())
    .await?)
}

pub async fn create_user(
    state: &AppStateV2,
    new_user: NewUser,
    default_role_id: i32,
) -> Result<(), AppError> {
    let mut tx = state.get_pool().begin().await?;

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

pub async fn get_email_by_id(state: &AppStateV2, user_id: i64) -> Result<String, AppError> {
    let (email,): (String,) =
        sqlx::query_as("SELECT email FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(state.get_pool())
            .await?;
    Ok(email)
}

pub async fn set_user_roles(
    state: &AppStateV2,
    user_id: i64,
    role_ids: &[i32],
) -> Result<(), AppError> {
    let mut tx = state.get_pool().begin().await?;

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

    for &role_id in role_ids {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(role_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
