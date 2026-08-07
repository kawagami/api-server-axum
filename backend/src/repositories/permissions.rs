use crate::{errors::AppError, structs::roles::Permission};
use sqlx::{Pool, Postgres};

pub async fn get_permissions(pool: &Pool<Postgres>) -> Result<Vec<Permission>, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, resource, action, description FROM permissions ORDER BY resource, action",
    )
    .fetch_all(pool)
    .await?)
}

/// 依 permission id 批次取 `resource:action` 字串（格式與 `AuthenticatedUser.permissions` 一致，
/// 才比對得起來）。不存在的 id 不會出現在結果裡 —— 呼叫端問的是「這批 id 代表哪些權限」，
/// 而不存在的 id 本來就授不出任何東西。
pub async fn permission_strings_by_ids(
    pool: &Pool<Postgres>,
    ids: &[i32],
) -> Result<Vec<String>, AppError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT resource, action FROM permissions WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().map(|(r, a)| format!("{r}:{a}")).collect())
}
