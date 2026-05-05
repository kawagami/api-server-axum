use crate::{
    errors::AppError,
    services::oauth as oauth_service,
    state::AppState,
    structs::members::{ExchangeCodeRequest, RefreshRequest, TokenResponse},
};
use axum::{
    extract::{Json, Path, State},
    routing::{get, post},
    Router,
};
use serde::Serialize;

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/{provider}", get(get_oauth_url))
        .route("/{provider}/exchange", post(exchange_code))
        .route("/refresh", post(refresh_token))
        .with_state(state)
}

#[derive(Serialize)]
struct OAuthUrlResponse {
    url: String,
}

async fn get_oauth_url(
    State(state): State<AppState>,
    Path(provider_str): Path<String>,
) -> Result<Json<OAuthUrlResponse>, AppError> {
    let provider = oauth_service::OAuthProvider::from_str(&provider_str)
        .ok_or_else(|| AppError::RequestError(crate::errors::RequestError::NotFound))?;

    let url = oauth_service::generate_oauth_url(&state, &provider).await?;
    Ok(Json(OAuthUrlResponse { url }))
}

async fn exchange_code(
    State(state): State<AppState>,
    Path(provider_str): Path<String>,
    Json(payload): Json<ExchangeCodeRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let provider = oauth_service::OAuthProvider::from_str(&provider_str)
        .ok_or_else(|| AppError::RequestError(crate::errors::RequestError::NotFound))?;

    let (access_token, refresh_token) =
        oauth_service::exchange_code(&state, &provider, &payload.code, &payload.state).await?;

    Ok(Json(TokenResponse { access_token, refresh_token }))
}

async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let (access_token, refresh_token) =
        oauth_service::refresh_member_token(&state, &payload.refresh_token).await?;

    Ok(Json(TokenResponse { access_token, refresh_token }))
}
