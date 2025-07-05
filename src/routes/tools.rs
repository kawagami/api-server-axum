use crate::errors::AppError;
use crate::structs::tools::{CompleteTimeResponse, Troops};
use crate::{state::AppStateV2, structs::tools::Params};
use axum::{extract::Query, routing::get, Json, Router};
use chrono::{Duration, Local};
use rand::{distributions::Alphanumeric, Rng};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/new_password", get(new_password))
        .route("/caculate_complete_time", get(caculate_complete_time))
}

pub async fn new_password(Query(params): Query<Params>) -> Result<Json<Vec<String>>, AppError> {
    let mut rng = rand::thread_rng();

    // 生成指定數量的隨機字串
    let result = (0..params.count)
        .map(|_| {
            (0..params.length)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect()
        })
        .collect();

    Ok(Json(result))
}

pub async fn caculate_complete_time(
    Query(troops): Query<Troops>,
) -> Result<Json<CompleteTimeResponse>, AppError> {
    let remaining_time = (troops.full - troops.now - troops.remaining_troops).max(0); // 跟 0 比取大者
    let minutes = remaining_time / 127;
    let complete_time = Local::now() + Duration::minutes(minutes);

    Ok(Json(CompleteTimeResponse {
        complete_time: complete_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        minutes,
    }))
}
