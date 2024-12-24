use crate::{errors::internal_error, repositories::redis, state::AppStateV2};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json, Response},
};
use rand::{distributions::Alphanumeric, Rng};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    count: Option<u8>,
    length: Option<u8>,
}

pub async fn using_connection_pool_extractor(
    State(state): State<AppStateV2>,
) -> Result<String, impl IntoResponse> {
    let pool = state.get_pool();

    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}

pub async fn for_test(State(state): State<AppStateV2>) -> Result<Json<Vec<String>>, Response> {
    let result = redis::redis_zrevrange(&state, "online_members")
        .await
        .unwrap();

    Ok(result)
}

pub async fn new_password(Query(params): Query<Params>) -> Result<Json<Vec<String>>, ()> {
    Ok(Json(generate_random_strings(params.length, params.count)))
}

fn generate_random_strings(length: Option<u8>, count: Option<u8>) -> Vec<String> {
    // 預設長度為 8，預設數量為 1
    let len = length.unwrap_or(8) as usize;
    let cnt = count.unwrap_or(1);

    let mut rng = rand::thread_rng();

    // 生成指定數量的隨機字串
    (0..cnt)
        .map(|_| (0..len).map(|_| rng.sample(Alphanumeric) as char).collect())
        .collect()
}
