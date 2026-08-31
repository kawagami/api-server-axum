use crate::errors::AppError;
use crate::extract::Json;
use crate::middleware::rate_limit;
use crate::services::tools as tools_service;
use crate::state::AppState;
use crate::structs::tools::{ConvertTextRequest, ConvertTextResponse};
use axum::{middleware, routing::post, Router};

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/convert_text", post(convert_text))
        .layer(middleware::from_fn_with_state(
            state,
            rate_limit::tools_rate_limit,
        ))
}

pub async fn convert_text(
    Json(req): Json<ConvertTextRequest>,
) -> Result<Json<ConvertTextResponse>, AppError> {
    let converted_text = tools_service::convert_text(req.text, req.direction).await?;
    Ok(Json(ConvertTextResponse { converted_text }))
}
