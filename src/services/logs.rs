use crate::{
    errors::AppError,
    repositories::logs::{self as logs_repo, Log},
};
use sqlx::{Pool, Postgres};

pub async fn get_logs(
    pool: &Pool<Postgres>,
    level: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Log>, AppError> {
    Ok(logs_repo::get_logs(pool, level, limit, offset).await?)
}
