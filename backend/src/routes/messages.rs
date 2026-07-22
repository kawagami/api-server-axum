use crate::{
    errors::AppError,
    middleware::rate_limit,
    services::messages as messages_service,
    state::AppState,
    structs::messages::{Message, NewMessage},
};
use axum::{extract::State, http::StatusCode, middleware, routing::post, Json, Router};

/// 公開端:訪客留言給站長(不需登入)。掛 per-IP rate limit 防灌水。
pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_message))
        .layer(middleware::from_fn_with_state(
            state,
            rate_limit::messages_rate_limit,
        ))
}

async fn create_message(
    State(state): State<AppState>,
    Json(input): Json<NewMessage>,
) -> Result<(StatusCode, Json<Message>), AppError> {
    let message = messages_service::create(state.get_pool(), input).await?;
    Ok((StatusCode::CREATED, Json(message)))
}
