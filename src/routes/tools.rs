use crate::{state::AppStateV2, structs::tools::Params};
use axum::{extract::Query, routing::get, Json, Router};
use rand::{distributions::Alphanumeric, Rng};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/new_password", get(new_password))
}

pub async fn new_password(Query(params): Query<Params>) -> Result<Json<Vec<String>>, ()> {
    // 預設長度為 8，預設數量為 1
    let len = params.length;
    let cnt = params.count;

    let mut rng = rand::thread_rng();

    // 生成指定數量的隨機字串
    let result = (0..cnt)
        .map(|_| (0..len).map(|_| rng.sample(Alphanumeric) as char).collect())
        .collect();

    Ok(Json(result))
}
