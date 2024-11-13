use crate::{errors::internal_error, state::AppStateV2};
use axum::{
    extract::State,
    response::{IntoResponse, Json, Response},
};
use rand::{distributions::Alphanumeric, Rng};

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
    // let redis_pool = state.get_redis_pool().await;
    // let mut conn = redis_pool.get().await.unwrap();

    // // 使用 zrange 取得範圍資料，並解析為 Vec<String>
    // let result: Vec<String> = conn.zrange("iszadd", 0, -1).await.unwrap();

    let _ = generate_random_string();
    // let _ = state.redis_zadd("iszadd", &value).await;

    // let _ = state.redis_zrem("iszadd", "O7lznz5D").await;

    // let result = state.redis_zrange("iszadd").await.unwrap();
    let result = state.redis_zrevrange("online_members").await.unwrap();

    Ok(result)
}

fn generate_random_string() -> String {
    let mut rng = rand::thread_rng();
    (0..8).map(|_| rng.sample(Alphanumeric) as char).collect()
}
