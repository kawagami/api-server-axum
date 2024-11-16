mod blogs;
mod firebase;
mod hackmd_note_list_tags;
mod hackmd_note_lists;
mod root;
mod ws;

use crate::{auth, state::AppStateV2};
use axum::{
    extract::DefaultBodyLimit,
    http::{header::CONTENT_TYPE, Method, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;

pub async fn app() -> Router {
    let origins = ["https://sg-vite.kawa.homes".parse().unwrap()];

    let state2 = AppStateV2::new().await;

    Router::new()
        .route("/", get(root::using_connection_pool_extractor))
        .route("/test", get(root::for_test))
        .route("/new_password", get(root::new_password))
        .route("/note_lists/:id", get(hackmd_note_lists::get_note_list))
        .route("/note_lists", get(hackmd_note_lists::get_all_note_lists))
        .route(
            "/note_list_tags",
            get(hackmd_note_list_tags::get_all_note_list_tags),
        )
        .route("/blogs/:id", get(blogs::get_blog))
        .route("/blogs", get(blogs::get_blogs))
        .route("/jwt", get(auth::sign_in))
        .route(
            "/firebase",
            get(firebase::images)
                .post(firebase::upload)
                .layer(middleware::from_fn(auth::authorize)),
        )
        .route("/ws", get(ws::websocket_handler))
        .route("/ws/messages", get(ws::ws_message))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1000 * 1000))
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
        .with_state(state2)
        .fallback(handler_404)
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
