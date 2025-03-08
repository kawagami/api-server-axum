mod auth;
mod blogs;
mod firebase;
mod notes;
mod root;
mod tools;
mod users;
mod ws;

use crate::{scheduler::initialize_scheduler, state::AppStateV2};
use axum::{
    extract::DefaultBodyLimit,
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    },
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;

pub async fn app() -> Router {
    let origins = ["https://kawa.homes".parse().unwrap()];
    let state = AppStateV2::new().await;

    let _scheduler = initialize_scheduler(state.clone()).await;

    Router::new()
        .merge(root::new())
        .nest("/jwt", auth::new())
        .nest("/firebase", firebase::new(state.clone()))
        .nest("/ws", ws::new())
        .nest("/blogs", blogs::new())
        .nest("/users", users::new())
        .nest("/tools", tools::new())
        .nest("/notes", notes::new())
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1000 * 1000))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(origins)
                .allow_headers([AUTHORIZATION, CONTENT_TYPE]),
        )
        .with_state(state)
        .fallback(root::handler_404)
}
