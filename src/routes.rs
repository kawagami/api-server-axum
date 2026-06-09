mod admin;
mod app_settings;
mod admin_blogs;
mod audit_logs;
mod auth;
mod blogs;
mod logs;
mod images;
mod members;
mod notes;
mod oauth;
mod permissions;
mod portfolio;
mod roles;
mod roster;
mod stocks;
mod tools;
mod users;
mod ws;

use crate::{logging::LogEntry, middleware::audit, scheduler::initialize_scheduler, state::AppState};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method, StatusCode},
    middleware,
    Router,
};
use tokio::sync::mpsc;
use tower_http::cors::AllowOrigin;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub(super) fn with_auth(state: AppState, router: Router<AppState>) -> Router<AppState> {
    router.layer(middleware::from_fn_with_state(
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

    let cors_origins: Vec<HeaderValue> = state
        .get_settings()
        .get("cors_allowed_origins")
        .unwrap_or_else(|| "https://kawa.homes".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    tokio::spawn(crate::logging::log_writer(log_rx, state.get_pool().clone()));

    initialize_scheduler(state.clone()).await;

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
        .nest("/oauth", oauth::new(state.clone()))
        .nest("/logs", logs::new(state.clone()))
        .nest_service("/uploads", ServeDir::new(&upload_path))
        .layer(middleware::from_fn_with_state(state.clone(), audit::audit_log))
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
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        )
        .with_state(state)
        .fallback(|| async { (StatusCode::NOT_FOUND, "empty page") })
}
