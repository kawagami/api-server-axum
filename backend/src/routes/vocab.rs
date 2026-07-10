use crate::{
    errors::{AppError, AuthError},
    middleware::auth,
    services::vocab as vocab_service,
    state::AppState,
    structs::{
        members::AuthenticatedMember,
        vocab::{
            AnswerRequest, AnswerResponse, Language, LeaderboardPeriod, LeaderboardResponse,
            MistakeEntry, StartRunRequest, StartRunResponse, VocabMe,
        },
    },
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

// 對局端點訪客也能用(選擇性驗證);me / mistakes / 複習模式仍需 member
pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/runs", post(start_run))
        .route("/runs/{id}/answer", post(answer))
        .route("/runs/{id}/finish", post(finish))
        .route("/me", get(me))
        .route("/mistakes", get(mistakes))
        .route("/leaderboard", get(leaderboard))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_member_optional,
        ))
}

/// 選擇性驗證下取出 member_id(訪客為 None)
fn caller(member: Option<Extension<AuthenticatedMember>>) -> Option<i64> {
    member.map(|Extension(m)| m.member_id)
}

/// me / mistakes 的題庫語言 query(?language=ja),缺省 en
#[derive(serde::Deserialize, Default)]
struct LangQuery {
    #[serde(default)]
    language: Language,
}

async fn start_run(
    member: Option<Extension<AuthenticatedMember>>,
    State(state): State<AppState>,
    body: Option<Json<StartRunRequest>>,
) -> Result<(StatusCode, Json<StartRunResponse>), AppError> {
    let req = body.map(|Json(b)| b).unwrap_or_default();
    let res = vocab_service::start_run(
        &state,
        caller(member),
        req.mode,
        req.language,
        req.duration_minutes,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(res)))
}

async fn finish(
    member: Option<Extension<AuthenticatedMember>>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AnswerResponse>, AppError> {
    Ok(Json(vocab_service::finish(&state, id, caller(member)).await?))
}

async fn answer(
    member: Option<Extension<AuthenticatedMember>>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<AnswerRequest>,
) -> Result<Json<AnswerResponse>, AppError> {
    Ok(Json(
        vocab_service::answer(&state, id, caller(member), &req).await?,
    ))
}

/// 排行榜 query(?language=ja&period=monthly),缺省 en + weekly
#[derive(serde::Deserialize, Default)]
struct LeaderboardQuery {
    #[serde(default)]
    language: Language,
    #[serde(default)]
    period: LeaderboardPeriod,
}

/// 訪客也能看榜;登入時額外回自己的名次
async fn leaderboard(
    member: Option<Extension<AuthenticatedMember>>,
    State(state): State<AppState>,
    Query(q): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    Ok(Json(
        vocab_service::leaderboard(&state, caller(member), q.language, q.period).await?,
    ))
}

async fn mistakes(
    member: Option<Extension<AuthenticatedMember>>,
    State(state): State<AppState>,
    Query(q): Query<LangQuery>,
) -> Result<Json<Vec<MistakeEntry>>, AppError> {
    let mid = caller(member).ok_or(AppError::AuthError(AuthError::Unauthorized))?;
    Ok(Json(vocab_service::mistakes(&state, mid, q.language).await?))
}

async fn me(
    member: Option<Extension<AuthenticatedMember>>,
    State(state): State<AppState>,
    Query(q): Query<LangQuery>,
) -> Result<Json<VocabMe>, AppError> {
    let mid = caller(member).ok_or(AppError::AuthError(AuthError::Unauthorized))?;
    Ok(Json(vocab_service::me(&state, mid, q.language).await?))
}
