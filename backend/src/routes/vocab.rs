use crate::{
    errors::AppError,
    middleware::auth,
    services::vocab as vocab_service,
    state::AppState,
    structs::{
        members::AuthenticatedMember,
        vocab::{
            AnswerRequest, AnswerResponse, MistakeEntry, StartRunRequest, StartRunResponse, VocabMe,
        },
    },
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/runs", post(start_run))
        .route("/runs/{id}/answer", post(answer))
        .route("/me", get(me))
        .route("/mistakes", get(mistakes))
        .layer(middleware::from_fn_with_state(state, auth::authorize_member))
}

async fn start_run(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    body: Option<Json<StartRunRequest>>,
) -> Result<(StatusCode, Json<StartRunResponse>), AppError> {
    let mode = body.map(|Json(b)| b.mode).unwrap_or_default();
    let res = vocab_service::start_run(&state, auth_member.member_id, mode).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

async fn mistakes(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<Vec<MistakeEntry>>, AppError> {
    Ok(Json(
        vocab_service::mistakes(&state, auth_member.member_id).await?,
    ))
}

async fn answer(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AnswerRequest>,
) -> Result<Json<AnswerResponse>, AppError> {
    Ok(Json(
        vocab_service::answer(&state, id, auth_member.member_id, &req).await?,
    ))
}

async fn me(
    Extension(auth_member): Extension<AuthenticatedMember>,
    State(state): State<AppState>,
) -> Result<Json<VocabMe>, AppError> {
    Ok(Json(vocab_service::me(&state, auth_member.member_id).await?))
}
