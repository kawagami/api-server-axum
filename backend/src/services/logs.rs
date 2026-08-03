use crate::{
    errors::AppError,
    repositories::logs::{self as logs_repo, Log},
    structs::{logs::LogQuery, pagination::Paginated},
};
use sqlx::{Pool, Postgres};

pub async fn get_logs(
    pool: &Pool<Postgres>,
    filter: &LogQuery,
    limit: i64,
    offset: i64,
) -> Result<Paginated<Log>, AppError> {
    // count 與 list 併發跑：序列 await 是白吃一倍延遲（範本同 services/blogs.rs）
    let (data, total) = tokio::try_join!(
        logs_repo::get_logs(pool, filter, limit, offset),
        logs_repo::count_logs(pool, filter),
    )?;

    Ok(Paginated::new(data, total))
}

pub async fn logs_by_request(
    pool: &Pool<Postgres>,
    request_id: &str,
) -> Result<Vec<Log>, AppError> {
    Ok(logs_repo::logs_by_request(pool, request_id).await?)
}
