use crate::errors::{AppError, RequestError, SystemError};
use crate::middleware::rate_limit;
use crate::structs::tools::{
    CompleteTimeResponse, ConvertTextRequest, ConvertTextResponse, ConversionDirection, Troops,
};
use crate::{state::AppState, structs::tools::Params};
use axum::{extract::Query, middleware, routing::get, routing::post, Json, Router};
use chrono::{Duration, Local};
use rand::{distr::Alphanumeric, Rng};
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
    let mut rng = rand::rng();

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

/// 轉換輸入的長度上限。全域 body 上限是 10MB（routes.rs 的 RequestBodyLimitLayer），
/// 對「貼一篇文章來轉」這個用途來說過寬 —— 一篇長文的中文約 50–100KB，256KB 綽綽有餘。
///
/// 實測 zhconv 0.4.1 的 CPU 成本並不高（本機 10MB ≈ 28ms、1MB ≈ 2.5ms），所以這個上限
/// 主要是擋記憶體：1 核 1G 的 VPS 上，10MB 的 body 加上同樣大的輸出與中間配置，
/// 幾個併發請求就很有感。CPU 那一面交給下面的 spawn_blocking。
const CONVERT_TEXT_MAX_BYTES: usize = 256 * 1024;

pub async fn convert_text(
    Json(req): Json<ConvertTextRequest>,
) -> Result<Json<ConvertTextResponse>, AppError> {
    if req.text.len() > CONVERT_TEXT_MAX_BYTES {
        return Err(RequestError::UnprocessableContent(format!(
            "text 長度上限為 {} KB",
            CONVERT_TEXT_MAX_BYTES / 1024
        ))
        .into());
    }

    let variant = match req.direction {
        ConversionDirection::T2s => Variant::ZhCN,
        ConversionDirection::S2t => Variant::ZhHant,
    };

    // zhconv 是同步 CPU 工作，直接跑會佔住 async worker（1 核機上就是全站一起等）。
    // 慣例同 services/auth.rs 的 bcrypt 與 services/images.rs 的 process_image。
    let text = req.text;
    let (original_text, converted_text) = tokio::task::spawn_blocking(move || {
        let converted = zhconv(&text, variant);
        (text, converted)
    })
    .await
    .map_err(|e| SystemError::Internal(format!("簡繁轉換執行失敗: {e}")))?;

    Ok(Json(ConvertTextResponse {
        original_text,
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
