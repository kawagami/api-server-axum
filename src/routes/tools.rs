use crate::errors::AppError;
use crate::middleware::rate_limit;
use crate::structs::tools::{
    CompleteTimeResponse, ConvertTextRequest, ConvertTextResponse, ConversionDirection, Troops,
};
use crate::{state::AppState, structs::tools::Params};
use axum::{extract::Query, middleware, routing::get, routing::post, Json, Router};
use chrono::{Duration, Local};
use rand::{distributions::Alphanumeric, Rng};
use zhconv::{zhconv, Variant};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/new_password", get(new_password))
        .route("/caculate_complete_time", get(caculate_complete_time))
        .route("/convert_text", post(convert_text))
        .layer(middleware::from_fn_with_state(
            state,
            rate_limit::tools_rate_limit,
        ))
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

pub async fn convert_text(
    Json(req): Json<ConvertTextRequest>,
) -> Result<Json<ConvertTextResponse>, AppError> {
    let variant = match req.direction {
        ConversionDirection::T2s => Variant::ZhCN,
        ConversionDirection::S2t => Variant::ZhHant,
    };
    let converted_text = zhconv(&req.text, variant);
    Ok(Json(ConvertTextResponse {
        original_text: req.text,
        converted_text,
    }))
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
