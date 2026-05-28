use crate::{errors::AppError, structs::app_settings::AppSetting};
use sqlx::{Pool, Postgres};

pub async fn get_all(pool: &Pool<Postgres>) -> Result<Vec<AppSetting>, AppError> {
    Ok(
        sqlx::query_as("SELECT key, value, description FROM app_settings ORDER BY key")
            .fetch_all(pool)
            .await?,
    )
}

pub async fn update(pool: &Pool<Postgres>, key: &str, value: &str) -> Result<AppSetting, AppError> {
    Ok(
        sqlx::query_as(
            "UPDATE app_settings SET value = $2 WHERE key = $1 RETURNING key, value, description",
        )
        .bind(key)
        .bind(value)
        .fetch_one(pool)
        .await?,
    )
}
