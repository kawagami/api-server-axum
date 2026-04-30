use crate::{errors::AppError, state::AppStateV2, structs::roles::Permission};

pub async fn get_permissions(state: &AppStateV2) -> Result<Vec<Permission>, AppError> {
    Ok(sqlx::query_as(
        "SELECT id, resource, action, description FROM permissions ORDER BY resource, action",
    )
    .fetch_all(state.get_pool())
    .await?)
}
