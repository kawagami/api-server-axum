use crate::{errors::internal_error, state::AppStateV2};
use axum::{extract::State, response::IntoResponse};

pub async fn using_connection_pool_extractor(
    State(state): State<AppStateV2>,
) -> Result<String, impl IntoResponse> {
    let pool = state.get_pool();

    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}
