use crate::{errors::AppError, structs::roles::Permission};
use sqlx::{Pool, Postgres};

pub async fn get_permissions(pool: &Pool<Postgres>) -> Result<Vec<Permission>, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, resource, action, description FROM permissions ORDER BY resource, action",
    )
    .fetch_all(pool)
    .await?)
}
