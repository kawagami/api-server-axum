mod auth;
mod blogs;
mod images;
mod notes;
mod root;
mod roster;
mod stocks;
mod tools;
mod users;
mod ws;

use crate::{scheduler::initialize_scheduler, state::AppStateV2};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    Router,
};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub async fn app() -> Router {
    let origins = ["https://kawa.homes".parse().unwrap()];
    let state = AppStateV2::new().await;

    let _scheduler = initialize_scheduler(state.clone()).await;

    let upload_path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string());

    Router::new()
        .merge(root::new())
        .nest("/jwt", auth::new())
        .nest("/blogs", blogs::new())
        .nest("/users", users::new(state.clone()))
        .nest("/tools", tools::new())
        .nest("/notes", notes::new())
        .nest("/stocks", stocks::new(state.clone()))
        .nest("/ws", ws::new())
        .nest("/roster", roster::new())
        .nest("/images", images::new(state.clone()))
        .nest_service("/uploads", ServeDir::new(&upload_path))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1000 * 1000))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(origins)
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]),
        )
        .with_state(state)
        .fallback(root::handler_404)
}
