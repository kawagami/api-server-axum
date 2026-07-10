mod admin;
mod admin_games;
mod admin_gov_tenders;
mod admin_invoice_lottery;
mod admin_stats;
mod admin_vocab;
mod app_settings;
mod admin_blogs;
mod audit_logs;
mod auth;
mod blogs;
mod logs;
mod images;
mod invoices;
mod ledger;
mod lotto;
mod members;
mod notes;
mod oauth;
mod permissions;
mod portfolio;
mod roles;
mod roster;
mod stocks;
mod tools;
mod torrents;
mod users;
mod vocab;
mod ws;

use crate::{logging::LogEntry, scheduler::initialize_scheduler, state::AppState};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method, StatusCode},
    middleware,
    Router,
};
use tokio::sync::mpsc;
use tower_http::cors::AllowOrigin;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub(super) fn with_auth(state: AppState, router: Router<AppState>) -> Router<AppState> {
    // audit 掛在 auth 內層：auth 先跑塞入 AuthenticatedUser，audit 直接讀 extension，不重複 decode JWT
    router
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::audit::audit_log,
        ))
        .layer(middleware::from_fn_with_state(
            state,
            crate::middleware::auth::authorize_and_load,
        ))
}

pub async fn app(log_rx: mpsc::Receiver<LogEntry>) -> Router {
    let state = AppState::new().await;

    sqlx::migrate!("./migrations")
        .run(state.get_pool())
        .await
        .expect("migration failed");

    state.reload_settings().await;

    crate::services::oauth::OAuthProvider::warn_if_partially_configured(
        state.get_config(),
        &state.get_settings(),
    );

    let cors_origins: Vec<HeaderValue> = state
        .get_settings()
        .get("cors_allowed_origins")
        .unwrap_or_else(|| "https://kawa.homes".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    tokio::spawn(crate::logging::log_writer(log_rx, state.get_pool().clone()));

    initialize_scheduler(state.clone()).await;

    // 重啟 resume：把 pending / downloading 的 torrent 補回 session
    tokio::spawn(crate::services::torrents::sync_active(state.clone()));

    // 遊戲計時掃描：偵測行棋方時鐘耗盡卻無人走步 → 主動判負（每遊戲一個 watcher）
    for hub in state.games().all() {
        hub.spawn_watcher(state.clone());
    }

    let upload_path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string());

    Router::new()
        .nest("/admin", admin::new(state.clone()))
        .nest("/blogs", blogs::new())
        .nest("/tools", tools::new(state.clone()))
        .nest("/notes", notes::new())
        .nest("/ws", ws::new(state.clone()))
        .nest("/roster", roster::new())
        .nest("/members", members::new(state.clone()))
        .nest("/member/portfolio", portfolio::new(state.clone()))
        .nest("/member/ledger", ledger::new(state.clone()))
        .nest("/member/invoices", invoices::new(state.clone()))
        .nest("/member/lotto", lotto::new(state.clone()))
        .nest("/member/vocab", vocab::new(state.clone()))
        .nest("/oauth", oauth::new(state.clone()))
        .nest("/logs", logs::new(state.clone()))
        .nest("/settings", app_settings::public())
        .nest_service("/uploads", ServeDir::new(&upload_path))
        // fallback 需在 layer 之前註冊，否則不會被下面的 request_id / TraceLayer 包住（404 也要有追蹤 id）
        .fallback(|| async { (StatusCode::NOT_FOUND, "empty page") })
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1000 * 1000))
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::PATCH,
                    Method::DELETE,
                ])
                .allow_origin(AllowOrigin::list(cors_origins))
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                // 讓瀏覽器端 JS 可讀到追蹤 id，方便回報問題時附上
                .expose_headers([header::HeaderName::from_static("x-request-id")]),
        )
        // 每請求一條 span：method / path / status / 延遲，並帶上 request_id（由下方 middleware 塞入 extensions）
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &axum::http::Request<_>| {
                let request_id = req
                    .extensions()
                    .get::<crate::middleware::request_id::RequestId>()
                    .map(|r| r.0.as_str())
                    .unwrap_or("-");
                tracing::info_span!(
                    "request",
                    method = %req.method(),
                    path = %req.uri().path(),
                    request_id = %request_id,
                )
            }),
        )
        // 最外層：產生 request_id → 供上面 span 讀取、寫回 response header、供錯誤 body 回溯
        .layer(middleware::from_fn(
            crate::middleware::request_id::request_id,
        ))
        .with_state(state)
}
