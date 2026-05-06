mod auth;
mod blogs;
mod images;
mod members;
mod notes;
mod oauth;
mod permissions;
mod roles;
mod root;
mod roster;
mod stocks;
mod tools;
mod users;
mod ws;

use crate::{scheduler::initialize_scheduler, state::AppState};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub async fn app() -> Router {
    let origins = ["https://kawa.homes".parse().unwrap()];
    let state = AppState::new().await;

    let _scheduler = initialize_scheduler(state.clone()).await;

    let upload_path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string());

    Router::new()
        .merge(root::new())
        .nest("/jwt", auth::new(state.clone()))
        .nest("/blogs", blogs::new())
        .nest("/users", users::new(state.clone()))
        .nest("/tools", tools::new())
        .nest("/notes", notes::new())
        .nest("/stocks", stocks::new(state.clone()))
        .nest("/ws", ws::new(state.clone()))
        .nest("/roster", roster::new())
        .nest("/images", images::new(state.clone()))
        .nest("/roles", roles::new(state.clone()))
        .nest("/permissions", permissions::new(state.clone()))
        .nest("/members", members::new(state.clone()))
        .nest("/auth", oauth::new(state.clone()))
        .nest_service("/uploads", ServeDir::new(&upload_path))
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
                .allow_origin(origins)
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        )
        .with_state(state)
        .fallback(root::handler_404)
}
