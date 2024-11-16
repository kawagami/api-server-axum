use crate::{errors::internal_error, state::AppStateV2};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json, Response},
};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    length: Option<u8>,
}

pub async fn using_connection_pool_extractor(
    State(state): State<AppStateV2>,
) -> Result<String, impl IntoResponse> {
    let pool = state.get_pool().await;

    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}

pub async fn for_test(State(state): State<AppStateV2>) -> Result<Json<Vec<String>>, Response> {
    let result = state.redis_zrevrange("online_members").await.unwrap();

    Ok(result)
}

pub async fn new_password(Query(params): Query<Params>) -> Result<String, ()> {
    Ok(generate_random_string(params.length))
}

fn generate_random_string(length: Option<u8>) -> String {
    // 預設長度為 8
    let len = length.unwrap_or(8);
    let mut rng = rand::thread_rng();
    (0..len).map(|_| rng.sample(Alphanumeric) as char).collect()
}
