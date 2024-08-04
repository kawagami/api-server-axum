mod blogs;
mod firebase;
mod hackmd_note_list_tags;
mod hackmd_note_lists;
mod root;
mod ws;

use crate::state::AppState;
use axum::{
    http::{header::CONTENT_TYPE, Method},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

pub async fn app() -> Router {
    let origins = [
        "http://localhost:5173".parse().unwrap(),
        "https://sg-vite.kawa.homes".parse().unwrap(),
    ];

    let state = AppState::new().await;

    Router::new()
        .route("/", get(root::using_connection_pool_extractor))
        .route("/note_lists/:id", get(hackmd_note_lists::get_note_list))
        .route("/note_lists", get(hackmd_note_lists::get_all_note_lists))
        .route(
            "/note_list_tags",
            get(hackmd_note_list_tags::get_all_note_list_tags),
        )
        .route("/blogs/:id", get(blogs::get_blog))
        .route("/blogs", get(blogs::get_blogs))
        .route("/firebase/upload", post(firebase::upload))
        .route("/ws", get(ws::websocket_handler))
        .layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            //
            // pay attention that for some request types like posting content-type: application/json
            // it is required to add ".allow_headers([http::header::CONTENT_TYPE])"
            // or see this issue https://github.com/tokio-rs/axum/issues/849
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(origins)
                .allow_headers([CONTENT_TYPE]),
        )
        .with_state(Arc::new(Mutex::new(state)))
}
